//! Named query helpers — keep SQL out of the trait impl.

use crate::models::{StoredContractEvent, StoredTransaction};
use crate::StorageError;
use sqlx::PgPool;

/// Inserts a transaction row. Ignores conflicts on `hash` (idempotent).
pub async fn insert_transaction(
    pool: &PgPool,
    tx: &StoredTransaction,
) -> Result<(), StorageError> {
    sqlx::query(
        "INSERT INTO transactions (hash, ledger_sequence, source_account, fee, successful, created_at) VALUES ($1, $2, $3, $4, $5, $6) ON CONFLICT (hash) DO NOTHING",
    )
    .bind(&tx.hash)
    .bind(tx.ledger_sequence)
    .bind(&tx.source_account)
    .bind(tx.fee)
    .bind(tx.successful)
    .bind(tx.created_at)
    .execute(pool)
    .await?;

    Ok(())
}

/// Inserts a contract event row.
pub async fn insert_contract_event(
    pool: &PgPool,
    event: &StoredContractEvent,
) -> Result<(), StorageError> {
    sqlx::query(
        "INSERT INTO contract_events (contract_id, ledger_sequence, tx_hash, topics, data) VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(&event.contract_id)
    .bind(event.ledger_sequence)
    .bind(&event.tx_hash)
    .bind(&event.topics)
    .bind(&event.data)
    .execute(pool)
    .await?;

    Ok(())
}

/// Returns the highest indexed ledger sequence, or 0 if none.
pub async fn get_latest_ledger(pool: &PgPool) -> Result<u32, StorageError> {
    let value: i64 = sqlx::query_scalar("SELECT COALESCE(MAX(ledger_sequence), 0) FROM transactions")
        .fetch_one(pool)
        .await?;

    Ok(value as u32)
}
