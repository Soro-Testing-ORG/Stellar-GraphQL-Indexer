//! Named query helpers — keep SQL out of the trait impl.

use crate::models::{StoredContractEvent, StoredTransaction};
use crate::StorageError;
use sqlx::{PgPool, Postgres, Transaction};

/// Inserts a transaction row. Ignores conflicts on `hash` (idempotent).
pub async fn insert_transaction(pool: &PgPool, tx: &StoredTransaction) -> Result<(), StorageError> {
    insert_transaction_query(pool, tx).await
}

pub async fn insert_transaction_in(
    tx: &mut Transaction<'_, Postgres>,
    transaction: &StoredTransaction,
) -> Result<(), StorageError> {
    sqlx::query(
        "INSERT INTO transactions (hash, ledger_sequence, source_account, fee, successful, created_at) VALUES ($1, $2, $3, $4, $5, $6) ON CONFLICT (hash) DO NOTHING",
    )
    .bind(&transaction.hash)
    .bind(transaction.ledger_sequence)
    .bind(&transaction.source_account)
    .bind(transaction.fee)
    .bind(transaction.successful)
    .bind(transaction.created_at)
    .execute(&mut **tx)
    .await?;
    Ok(())
}

async fn insert_transaction_query(
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
    insert_contract_event_query(pool, event).await
}

pub async fn insert_contract_event_in(
    tx: &mut Transaction<'_, Postgres>,
    event: &StoredContractEvent,
) -> Result<(), StorageError> {
    sqlx::query(
        "INSERT INTO contract_events (event_index, contract_id, ledger_sequence, tx_hash, topics, data) VALUES ($1, $2, $3, $4, $5, $6) ON CONFLICT (tx_hash, event_index) DO NOTHING",
    )
    .bind(event.event_index)
    .bind(&event.contract_id)
    .bind(event.ledger_sequence)
    .bind(&event.tx_hash)
    .bind(&event.topics)
    .bind(&event.data)
    .execute(&mut **tx)
    .await?;

    Ok(())
}

async fn insert_contract_event_query(
    pool: &PgPool,
    event: &StoredContractEvent,
) -> Result<(), StorageError> {
    sqlx::query(
        "INSERT INTO contract_events (event_index, contract_id, ledger_sequence, tx_hash, topics, data) VALUES ($1, $2, $3, $4, $5, $6) ON CONFLICT (tx_hash, event_index) DO NOTHING",
    )
    .bind(event.event_index)
    .bind(&event.contract_id)
    .bind(event.ledger_sequence)
    .bind(&event.tx_hash)
    .bind(&event.topics)
    .bind(&event.data)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn list_transactions(
    pool: &PgPool,
    limit: i64,
    offset: i64,
) -> Result<Vec<StoredTransaction>, StorageError> {
    Ok(sqlx::query_as::<_, StoredTransaction>(
        "SELECT hash, ledger_sequence, source_account, fee, successful, created_at FROM transactions ORDER BY ledger_sequence DESC, hash ASC LIMIT $1 OFFSET $2",
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?)
}

pub async fn list_contract_events(
    pool: &PgPool,
    contract_id: &str,
    limit: i64,
    offset: i64,
) -> Result<Vec<StoredContractEvent>, StorageError> {
    Ok(sqlx::query_as::<_, StoredContractEvent>(
        "SELECT id, event_index, contract_id, ledger_sequence, tx_hash, topics, data FROM contract_events WHERE contract_id = $1 ORDER BY ledger_sequence DESC, tx_hash ASC, event_index ASC LIMIT $2 OFFSET $3",
    )
    .bind(contract_id)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?)
}

/// Returns the highest indexed ledger sequence, or 0 if none.
pub async fn get_latest_ledger(pool: &PgPool) -> Result<u32, StorageError> {
    let value: i64 =
        sqlx::query_scalar("SELECT COALESCE(MAX(sequence), 0) FROM ledger_checkpoints")
            .fetch_one(pool)
            .await?;

    u32::try_from(value).map_err(|_| StorageError::InvalidLedgerSequence(value))
}
