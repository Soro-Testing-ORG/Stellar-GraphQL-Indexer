//! Storage — Postgres persistence layer.
//!
//! All database access goes through [`Db`]. The [`StorageBackend`] trait
//! allows swapping the implementation (e.g. for an in-memory store in tests).

pub mod models;
pub mod queries;

use async_trait::async_trait;
use sqlx::PgPool;
use thiserror::Error;

use crate::models::{StoredContractEvent, StoredTransaction};

#[derive(Debug, Error)]
pub enum StorageError {
    #[error("database error: {0}")]
    Db(#[from] sqlx::Error),
    #[error("database migration error: {0}")]
    Migration(#[from] sqlx::migrate::MigrateError),
    #[error("record not found")]
    NotFound,
}

/// Trait abstracting all persistence operations.
/// Implement this to add alternative backends (e.g. in-memory for tests).
#[async_trait]
pub trait StorageBackend: Send + Sync {
    async fn insert_transaction(&self, tx: &StoredTransaction) -> Result<(), StorageError>;
    async fn insert_contract_event(&self, event: &StoredContractEvent) -> Result<(), StorageError>;
    async fn get_transaction(&self, hash: &str) -> Result<StoredTransaction, StorageError>;
    async fn get_latest_ledger(&self) -> Result<u32, StorageError>;
}

/// Postgres-backed storage using sqlx.
pub struct Db {
    #[allow(dead_code)]
    pool: PgPool,
}

impl Db {
    /// Connects to Postgres and runs pending migrations.
    pub async fn connect(database_url: &str) -> Result<Self, StorageError> {
        let pool = PgPool::connect(database_url).await?;
        sqlx::migrate!("../../migrations").run(&pool).await?;
        Ok(Self { pool })
    }
}

#[async_trait]
impl StorageBackend for Db {
    async fn insert_transaction(&self, tx: &StoredTransaction) -> Result<(), StorageError> {
        queries::insert_transaction(&self.pool, tx).await
    }

    async fn insert_contract_event(&self, event: &StoredContractEvent) -> Result<(), StorageError> {
        queries::insert_contract_event(&self.pool, event).await
    }

    async fn get_transaction(&self, hash: &str) -> Result<StoredTransaction, StorageError> {
        let row = sqlx::query_as::<_, StoredTransaction>(
            "SELECT hash, ledger_sequence, source_account, fee, successful, created_at FROM transactions WHERE hash = $1",
        )
        .bind(hash)
        .fetch_optional(&self.pool)
        .await?;

        row.ok_or(StorageError::NotFound)
    }

    async fn get_latest_ledger(&self) -> Result<u32, StorageError> {
        queries::get_latest_ledger(&self.pool).await
    }
}
