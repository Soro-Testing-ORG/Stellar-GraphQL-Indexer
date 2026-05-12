//! Storage — Postgres persistence layer.
//!
//! All database access goes through [`Db`]. The [`StorageBackend`] trait
//! allows swapping the implementation (e.g. for an in-memory store in tests).

pub mod models;
pub mod queries;

use async_trait::async_trait;
use sqlx::PgPool;
use thiserror::Error;

use crate::models::{StoredTransaction, StoredContractEvent};

#[derive(Debug, Error)]
pub enum StorageError {
    #[error("database error: {0}")]
    Db(#[from] sqlx::Error),
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
    ///
    /// # TODO
    /// Call `sqlx::migrate!("../../migrations").run(&pool)` after connecting.
    pub async fn connect(database_url: &str) -> Result<Self, StorageError> {
        let pool = PgPool::connect(database_url).await?;
        Ok(Self { pool })
    }
}

#[async_trait]
impl StorageBackend for Db {
    async fn insert_transaction(&self, tx: &StoredTransaction) -> Result<(), StorageError> {
        // TODO: INSERT INTO transactions (...) VALUES (...)
        let _ = tx;
        todo!("Db::insert_transaction: sqlx INSERT")
    }

    async fn insert_contract_event(&self, event: &StoredContractEvent) -> Result<(), StorageError> {
        // TODO: INSERT INTO contract_events (...) VALUES (...)
        let _ = event;
        todo!("Db::insert_contract_event: sqlx INSERT")
    }

    async fn get_transaction(&self, hash: &str) -> Result<StoredTransaction, StorageError> {
        // TODO: SELECT * FROM transactions WHERE hash = $1
        let _ = hash;
        todo!("Db::get_transaction: sqlx SELECT")
    }

    async fn get_latest_ledger(&self) -> Result<u32, StorageError> {
        // TODO: SELECT MAX(ledger_sequence) FROM transactions
        todo!("Db::get_latest_ledger: sqlx SELECT MAX")
    }
}
