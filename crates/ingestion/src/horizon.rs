//! Horizon HTTP client helpers.
//!
//! Reference: https://developers.stellar.org/api/horizon

use crate::IngestionError;

/// Fetches a single ledger record from Horizon by sequence number.
///
/// Returns the raw JSON response from `GET /ledgers/{sequence}`.
pub async fn get_ledger(
    client: &reqwest::Client,
    horizon_url: &str,
    sequence: u32,
) -> Result<serde_json::Value, IngestionError> {
    let url = format!("{}/ledgers/{}", horizon_url.trim_end_matches('/'), sequence);
    let resp = client.get(&url).send().await?.error_for_status()?;
    Ok(resp.json().await?)
}

/// Fetches all transactions for a given ledger sequence.
///
/// Pages through `GET /ledgers/{sequence}/transactions` until all records
/// are collected (Horizon returns max 200 per page).
pub async fn get_transactions(
    client: &reqwest::Client,
    horizon_url: &str,
    ledger_sequence: u32,
) -> Result<Vec<serde_json::Value>, IngestionError> {
    let base = horizon_url.trim_end_matches('/');
    let url = format!(
        "{}/ledgers/{}/transactions?limit=200&include_failed=true",
        base, ledger_sequence
    );

    let resp = client.get(&url).send().await?.error_for_status()?;
    let body: serde_json::Value = resp.json().await?;

    let records = body["_embedded"]["records"]
        .as_array()
        .cloned()
        .unwrap_or_default();

    Ok(records)
}
