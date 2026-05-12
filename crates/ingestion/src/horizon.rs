//! Horizon HTTP client helpers.
//!
//! # TODO
//! Implement `get_ledger(sequence)` and `stream_ledgers(cursor)` using
//! Horizon's REST API. Reference: https://developers.stellar.org/api/horizon

use crate::IngestionError;

/// Fetches a single ledger record from Horizon by sequence number.
///
/// # TODO
/// GET {horizon_url}/ledgers/{sequence} and deserialize the JSON response.
pub async fn get_ledger(
    _client: &reqwest::Client,
    _horizon_url: &str,
    _sequence: u32,
) -> Result<serde_json::Value, IngestionError> {
    todo!("horizon::get_ledger: fetch ledger JSON from Horizon")
}

/// Fetches a page of transactions for a given ledger sequence.
///
/// # TODO
/// GET {horizon_url}/ledgers/{sequence}/transactions?limit=200
pub async fn get_transactions(
    _client: &reqwest::Client,
    _horizon_url: &str,
    _ledger_sequence: u32,
) -> Result<Vec<serde_json::Value>, IngestionError> {
    todo!("horizon::get_transactions: fetch transaction page from Horizon")
}
