//! Named query helpers — keep SQL out of the trait impl.
//!
//! Each function here corresponds to one SQL statement.
//! # TODO
//! Implement each query using `sqlx::query_as!` macros.

use crate::models::{StoredContractEvent, StoredTransaction};
use crate::StorageError;
use sqlx::PgPool;

/// Inserts a transaction row. Ignores conflicts on `hash` (idempotent).
///
/// # TODO
/// ```sql
/// INSERT INTO transactions (hash, ledger_sequence, source_account, fee, successful, created_at)
/// VALUES ($1, $2, $3, $4, $5, $6)
/// ON CONFLICT (hash) DO NOTHING
/// ```
pub async fn insert_transaction(
    _pool: &PgPool,
    _tx: &StoredTransaction,
) -> Result<(), StorageError> {
    todo!("queries::insert_transaction")
}

/// Inserts a contract event row.
///
/// # TODO
/// ```sql
/// INSERT INTO contract_events (contract_id, ledger_sequence, tx_hash, topics, data)
/// VALUES ($1, $2, $3, $4, $5)
/// ```
pub async fn insert_contract_event(
    _pool: &PgPool,
    _event: &StoredContractEvent,
) -> Result<(), StorageError> {
    todo!("queries::insert_contract_event")
}

/// Returns the highest indexed ledger sequence, or 0 if none.
///
/// # TODO
/// ```sql
/// SELECT COALESCE(MAX(ledger_sequence), 0) FROM transactions
/// ```
pub async fn get_latest_ledger(_pool: &PgPool) -> Result<u32, StorageError> {
    todo!("queries::get_latest_ledger")
}
