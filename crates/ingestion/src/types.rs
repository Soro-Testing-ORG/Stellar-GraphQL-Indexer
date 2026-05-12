//! Shared data types produced by the ingestion layer.

use serde::{Deserialize, Serialize};

/// All indexed data from a single closed ledger.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LedgerBundle {
    pub sequence: u32,
    pub closed_at: i64,
    pub transactions: Vec<Transaction>,
    pub contract_events: Vec<ContractEvent>,
}

/// A Stellar transaction with its operations decoded.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub hash: String,
    pub ledger_sequence: u32,
    pub source_account: String,
    pub fee: u32,
    pub successful: bool,
    pub operations: Vec<Operation>,
}

/// A single operation within a transaction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Operation {
    pub id: u64,
    pub op_type: String,
    pub source_account: Option<String>,
    /// Raw operation body as JSON for flexibility.
    pub body: serde_json::Value,
}

/// A Soroban contract event emitted during a transaction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractEvent {
    pub contract_id: String,
    pub ledger_sequence: u32,
    pub tx_hash: String,
    /// Event topics decoded from XDR.
    pub topics: Vec<String>,
    /// Event data decoded from XDR.
    pub data: serde_json::Value,
}
