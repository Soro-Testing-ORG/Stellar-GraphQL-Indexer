//! GraphQL schema — types and resolvers.

use std::sync::Arc;

use async_graphql::{Context, EmptyMutation, EmptySubscription, Object, Schema, SimpleObject};
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
}

pub struct QueryRoot;

#[Object]
impl QueryRoot {
    async fn transaction(&self, ctx: &Context<'_>, hash: String) -> Option<Transaction> {
        let storage = ctx.data::<Arc<dyn StorageBackend>>().ok()?;
        match storage.get_transaction(&hash).await {
            Ok(tx) => Some(Transaction {
                hash: tx.hash,
                ledger_sequence: tx.ledger_sequence,
                source_account: tx.source_account,
                fee: tx.fee,
                successful: tx.successful,
            }),
            Err(StorageError::NotFound) => None,
            Err(_) => None,
        }
    }

    async fn transactions(
        &self,
        ctx: &Context<'_>,
        #[graphql(default = 20)] limit: i32,
        #[graphql(default = 0)] offset: i32,
    ) -> Vec<Transaction> {
        let _ = offset;
        let storage = match ctx.data::<Arc<dyn StorageBackend>>() {
            Ok(storage) => storage,
            Err(_) => return vec![],
        };

        let _ = storage;
        let _ = limit;
        vec![]
    }

    async fn contract_events(
        &self,
        ctx: &Context<'_>,
        contract_id: String,
        #[graphql(default = 20)] limit: i32,
    ) -> Vec<ContractEvent> {
        let _ = limit;
        let _ = contract_id;
        let storage = match ctx.data::<Arc<dyn StorageBackend>>() {
            Ok(storage) => storage,
            Err(_) => return vec![],
        };
        let _ = storage;
        vec![]
    }
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
