//! Ingestion — streams ledger closes from Stellar Horizon and decodes XDR.
//!
//! The main entry point is [`LedgerStream`], which yields [`LedgerBundle`]s
//! containing the decoded data for each closed ledger.

pub mod decoder;
pub mod horizon;
pub mod types;

pub use types::{ContractEvent, LedgerBundle, Operation, Transaction};

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
pub struct LedgerStream {
    pub horizon_url: String,
    pub cursor: u32,
    client: reqwest::Client,
}

impl LedgerStream {
    pub fn new(horizon_url: impl Into<String>, start_ledger: u32) -> Self {
        Self {
            horizon_url: horizon_url.into(),
            cursor: start_ledger,
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl LedgerSource for LedgerStream {
    async fn next_ledger(&mut self) -> Result<LedgerBundle, IngestionError> {
        let ledger = horizon::get_ledger(&self.client, &self.horizon_url, self.cursor).await?;

        let sequence = ledger["sequence"]
            .as_u64()
            .ok_or(IngestionError::Xdr("missing ledger sequence".into()))?
            as u32;

        let tx_records =
            horizon::get_transactions(&self.client, &self.horizon_url, sequence).await?;

        let transactions = tx_records
            .iter()
            .map(|record| decoder::decode_transaction(record, sequence))
            .collect::<Result<Vec<_>, _>>()?;

        let mut contract_events = Vec::new();
        for tx in &tx_records {
            let tx_hash = tx["hash"]
                .as_str()
                .ok_or_else(|| IngestionError::Xdr("transaction record missing hash".into()))?;
            if let Some(meta) = tx["result_meta_xdr"].as_str() {
                contract_events.extend(decoder::decode_contract_events(meta, sequence, tx_hash)?);
            }
        }

        let closed_at = ledger["closed_at"]
            .as_str()
            .and_then(|v| chrono::DateTime::parse_from_rfc3339(v).ok())
            .map(|dt| dt.timestamp())
            .or_else(|| ledger["closed_at"].as_i64())
            .ok_or_else(|| IngestionError::Xdr("missing or invalid ledger closed_at".into()))?;

        let bundle = LedgerBundle {
            sequence,
            closed_at,
            transactions,
            contract_events,
        };

        self.cursor = sequence + 1;
        Ok(bundle)
    }
}
