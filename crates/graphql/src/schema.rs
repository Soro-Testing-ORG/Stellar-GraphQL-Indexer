//! GraphQL schema — types and resolvers.

use std::sync::Arc;

use async_graphql::{
    Context, EmptyMutation, EmptySubscription, Error, Json, Object, Schema, SimpleObject,
};
use storage::{StorageBackend, StorageError};

/// GraphQL representation of an indexed transaction.
#[derive(SimpleObject)]
pub struct Transaction {
    pub hash: String,
    pub ledger_sequence: i32,
    pub source_account: String,
    pub fee: i32,
    pub successful: bool,
}

/// GraphQL representation of a Soroban contract event.
#[derive(SimpleObject)]
pub struct ContractEvent {
    pub contract_id: String,
    pub ledger_sequence: i32,
    pub tx_hash: String,
    pub topics: Vec<String>,
    pub data: Json<serde_json::Value>,
}

pub struct QueryRoot;

#[Object]
impl QueryRoot {
    async fn transaction(
        &self,
        ctx: &Context<'_>,
        hash: String,
    ) -> async_graphql::Result<Option<Transaction>> {
        let storage = ctx.data::<Arc<dyn StorageBackend>>()?;
        match storage.get_transaction(&hash).await {
            Ok(tx) => Ok(Some(Transaction {
                hash: tx.hash,
                ledger_sequence: tx.ledger_sequence,
                source_account: tx.source_account,
                fee: tx.fee,
                successful: tx.successful,
            })),
            Err(StorageError::NotFound) => Ok(None),
            Err(err) => Err(Error::new(err.to_string())),
        }
    }

    async fn transactions(
        &self,
        ctx: &Context<'_>,
        #[graphql(default = 20)] limit: i32,
        #[graphql(default = 0)] offset: i32,
    ) -> async_graphql::Result<Vec<Transaction>> {
        validate_pagination(limit, offset)?;
        let storage = ctx.data::<Arc<dyn StorageBackend>>()?;
        Ok(storage
            .list_transactions(i64::from(limit), i64::from(offset))
            .await
            .map_err(|err| Error::new(err.to_string()))?
            .into_iter()
            .map(|tx| Transaction {
                hash: tx.hash,
                ledger_sequence: tx.ledger_sequence,
                source_account: tx.source_account,
                fee: tx.fee,
                successful: tx.successful,
            })
            .collect())
    }

    async fn contract_events(
        &self,
        ctx: &Context<'_>,
        contract_id: String,
        #[graphql(default = 20)] limit: i32,
    ) -> async_graphql::Result<Vec<ContractEvent>> {
        validate_pagination(limit, 0)?;
        let storage = ctx.data::<Arc<dyn StorageBackend>>()?;
        Ok(storage
            .list_contract_events(&contract_id, i64::from(limit), 0)
            .await
            .map_err(|err| Error::new(err.to_string()))?
            .into_iter()
            .map(|event| ContractEvent {
                contract_id: event.contract_id,
                ledger_sequence: event.ledger_sequence,
                tx_hash: event.tx_hash,
                topics: event
                    .topics
                    .as_array()
                    .into_iter()
                    .flatten()
                    .map(|value| {
                        value
                            .as_str()
                            .map(str::to_owned)
                            .unwrap_or_else(|| value.to_string())
                    })
                    .collect(),
                data: Json(event.data),
            })
            .collect())
    }
}

fn validate_pagination(limit: i32, offset: i32) -> async_graphql::Result<()> {
    if !(1..=100).contains(&limit) {
        return Err(Error::new("limit must be between 1 and 100"));
    }
    if offset < 0 {
        return Err(Error::new("offset must not be negative"));
    }
    Ok(())
}

pub type IndexerSchema = Schema<QueryRoot, EmptyMutation, EmptySubscription>;

pub fn build_schema() -> IndexerSchema {
    Schema::build(QueryRoot, EmptyMutation, EmptySubscription).finish()
}

pub fn build_schema_with_storage(storage: Arc<dyn StorageBackend>) -> IndexerSchema {
    Schema::build(QueryRoot, EmptyMutation, EmptySubscription)
        .data(storage)
        .finish()
}

#[cfg(test)]
mod tests {
    use super::*;
    use storage::models::{StoredContractEvent, StoredTransaction};

    struct TestStorage {
        transactions: Vec<StoredTransaction>,
        events: Vec<StoredContractEvent>,
    }

    #[async_trait::async_trait]
    impl StorageBackend for TestStorage {
        async fn insert_transaction(&self, _: &StoredTransaction) -> Result<(), StorageError> {
            Ok(())
        }

        async fn insert_contract_event(&self, _: &StoredContractEvent) -> Result<(), StorageError> {
            Ok(())
        }

        async fn get_transaction(&self, hash: &str) -> Result<StoredTransaction, StorageError> {
            self.transactions
                .iter()
                .find(|tx| tx.hash == hash)
                .cloned()
                .ok_or(StorageError::NotFound)
        }

        async fn list_transactions(
            &self,
            limit: i64,
            offset: i64,
        ) -> Result<Vec<StoredTransaction>, StorageError> {
            Ok(self
                .transactions
                .iter()
                .skip(offset as usize)
                .take(limit as usize)
                .cloned()
                .collect())
        }

        async fn list_contract_events(
            &self,
            contract_id: &str,
            limit: i64,
            offset: i64,
        ) -> Result<Vec<StoredContractEvent>, StorageError> {
            Ok(self
                .events
                .iter()
                .filter(|event| event.contract_id == contract_id)
                .skip(offset as usize)
                .take(limit as usize)
                .cloned()
                .collect())
        }

        async fn get_latest_ledger(&self) -> Result<u32, StorageError> {
            Ok(0)
        }

        async fn persist_ledger(
            &self,
            _: u32,
            _: i64,
            _: &[StoredTransaction],
            _: &[StoredContractEvent],
        ) -> Result<(), StorageError> {
            Ok(())
        }
    }

    fn test_schema() -> IndexerSchema {
        build_schema_with_storage(Arc::new(TestStorage {
            transactions: vec![StoredTransaction {
                hash: "abc".to_string(),
                ledger_sequence: 42,
                source_account: "GABC".to_string(),
                fee: 100,
                successful: true,
                created_at: 123,
            }],
            events: vec![StoredContractEvent {
                id: 1,
                event_index: 0,
                contract_id: "CABC".to_string(),
                ledger_sequence: 42,
                tx_hash: "abc".to_string(),
                topics: serde_json::json!(["transfer"]),
                data: serde_json::json!({"amount": 12}),
            }],
        }))
    }

    #[tokio::test]
    async fn transaction_and_list_queries_return_storage_data() {
        let response = test_schema()
            .execute(
                "{ transaction(hash: \"abc\") { hash ledgerSequence } transactions(limit: 1) { hash } }",
            )
            .await;

        assert!(response.errors.is_empty(), "{:?}", response.errors);
        assert_eq!(
            response.data.into_json().unwrap(),
            serde_json::json!({
                "transaction": {"hash": "abc", "ledgerSequence": 42},
                "transactions": [{"hash": "abc"}]
            })
        );
    }

    #[tokio::test]
    async fn contract_event_query_returns_topics_and_payload() {
        let response = test_schema()
            .execute("{ contractEvents(contractId: \"CABC\") { txHash topics data } }")
            .await;

        assert!(response.errors.is_empty(), "{:?}", response.errors);
        assert_eq!(
            response.data.into_json().unwrap(),
            serde_json::json!({
                "contractEvents": [{
                    "txHash": "abc",
                    "topics": ["transfer"],
                    "data": {"amount": 12}
                }]
            })
        );
    }
}
