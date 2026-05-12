//! GraphQL schema — types and resolvers.

#![allow(unreachable_code)] // todo!() stubs in resolvers are intentional

use async_graphql::{EmptyMutation, EmptySubscription, Object, Schema, SimpleObject};

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
    /// Fetch a transaction by its hash.
    ///
    /// # TODO
    /// Call `storage.get_transaction(hash)` and map to `Transaction`.
    async fn transaction(&self, hash: String) -> Option<Transaction> {
        let _ = hash;
        todo!("QueryRoot::transaction: call storage backend")
    }

    /// List the most recent `limit` transactions (default 20, max 100).
    ///
    /// # TODO
    /// Call `storage.list_transactions(limit, offset)`.
    async fn transactions(
        &self,
        #[graphql(default = 20)] limit: i32,
        #[graphql(default = 0)] offset: i32,
    ) -> Vec<Transaction> {
        let _ = (limit, offset);
        todo!("QueryRoot::transactions: call storage backend")
    }

    /// List contract events for a given contract ID.
    ///
    /// # TODO
    /// Call `storage.list_contract_events(contract_id, limit, offset)`.
    async fn contract_events(
        &self,
        contract_id: String,
        #[graphql(default = 20)] limit: i32,
    ) -> Vec<ContractEvent> {
        let _ = (contract_id, limit);
        todo!("QueryRoot::contract_events: call storage backend")
    }
}

pub type IndexerSchema = Schema<QueryRoot, EmptyMutation, EmptySubscription>;

pub fn build_schema() -> IndexerSchema {
    Schema::build(QueryRoot, EmptyMutation, EmptySubscription).finish()
}
