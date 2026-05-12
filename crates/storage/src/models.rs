//! Database row types — map 1:1 to Postgres tables.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct StoredTransaction {
    pub hash: String,
    pub ledger_sequence: i32,
    pub source_account: String,
    pub fee: i32,
    pub successful: bool,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct StoredContractEvent {
    pub id: i64,
    pub contract_id: String,
    pub ledger_sequence: i32,
    pub tx_hash: String,
    pub topics: serde_json::Value,
    pub data: serde_json::Value,
}
