//! Ingestion — streams ledger closes from Stellar Horizon and decodes XDR.
//!
//! The main entry point is [`LedgerStream`], which yields [`LedgerBundle`]s
//! containing the decoded data for each closed ledger.

pub mod horizon;
pub mod decoder;
pub mod types;

pub use types::{LedgerBundle, Transaction, Operation, ContractEvent};

use async_trait::async_trait;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum IngestionError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("XDR decode error: {0}")]
    Xdr(String),
    #[error("Stream ended unexpectedly")]
    StreamEnded,
}

/// Streams ledger data from a Stellar data source.
///
/// Implement this trait to add support for new data sources
/// (e.g. Stellar Core database, Galexie, archive files).
#[async_trait]
pub trait LedgerSource: Send + Sync {
    /// Returns the next closed ledger bundle, blocking until one is available.
    async fn next_ledger(&mut self) -> Result<LedgerBundle, IngestionError>;
}

/// Streams ledgers from Horizon's `/ledgers` endpoint.
///
/// # TODO
/// Implement polling loop: GET /ledgers?order=asc&cursor={cursor},
/// decode each ledger's XDR envelope, yield a LedgerBundle.
pub struct LedgerStream {
    pub horizon_url: String,
    pub cursor: u32,
}

impl LedgerStream {
    pub fn new(horizon_url: impl Into<String>, start_ledger: u32) -> Self {
        Self {
            horizon_url: horizon_url.into(),
            cursor: start_ledger,
        }
    }
}

#[async_trait]
impl LedgerSource for LedgerStream {
    async fn next_ledger(&mut self) -> Result<LedgerBundle, IngestionError> {
        // TODO: poll Horizon /ledgers?cursor=self.cursor&order=asc&limit=1
        // TODO: decode response XDR via decoder::decode_ledger_close
        // TODO: advance self.cursor
        todo!("LedgerStream::next_ledger: poll Horizon and decode XDR")
    }
}
