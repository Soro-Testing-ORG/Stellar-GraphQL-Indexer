//! XDR decoding utilities.

use stellar_xdr::curr::{Limits, ReadXdr, TransactionEnvelope};

use crate::types::{ContractEvent, Operation, Transaction};
use crate::IngestionError;

/// Decodes a Horizon transaction JSON record into a [`Transaction`].
///
/// Horizon provides most fields as JSON; we decode `envelope_xdr` only
/// to extract the operations list.
pub fn decode_transaction(
    record: &serde_json::Value,
    ledger_sequence: u32,
) -> Result<Transaction, IngestionError> {
    let hash = record["hash"]
        .as_str()
        .ok_or_else(|| IngestionError::Xdr("missing hash".into()))?
        .to_string();

    let source_account = record["source_account"]
        .as_str()
        .unwrap_or("")
        .to_string();

    let fee = record["fee_charged"]
        .as_str()
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(0);

    let successful = record["successful"].as_bool().unwrap_or(false);

    let operations = if let Some(xdr_b64) = record["envelope_xdr"].as_str() {
        decode_operations(xdr_b64)?
    } else {
        vec![]
    };

    Ok(Transaction {
        hash,
        ledger_sequence,
        source_account,
        fee,
        successful,
        operations,
    })
}

/// Extracts operations from a base64-encoded `TransactionEnvelope` XDR string.
fn decode_operations(xdr_b64: &str) -> Result<Vec<Operation>, IngestionError> {
    let envelope = TransactionEnvelope::from_xdr_base64(xdr_b64, Limits::none())
        .map_err(|e: stellar_xdr::curr::Error| IngestionError::Xdr(e.to_string()))?;

    let ops = match &envelope {
        TransactionEnvelope::Tx(env) => env.tx.operations.as_slice(),
        TransactionEnvelope::TxV0(env) => env.tx.operations.as_slice(),
        TransactionEnvelope::TxFeeBump(env) => {
            // FeeBump wraps an inner Tx — extract its operations
            use stellar_xdr::curr::FeeBumpTransactionInnerTx;
            match &env.tx.inner_tx {
                FeeBumpTransactionInnerTx::Tx(inner) => inner.tx.operations.as_slice(),
            }
        }
    };

    Ok(ops
        .iter()
        .enumerate()
        .map(|(i, op)| Operation {
            id: i as u64,
            op_type: op.body.name().to_string(),
            source_account: op.source_account.as_ref().map(|a| a.to_string()),
            body: serde_json::Value::String(op.body.name().to_string()),
        })
        .collect())
}

/// Decodes Soroban contract events from a `TransactionMeta` XDR string.
///
/// # TODO
/// 1. Parse `TransactionMeta` XDR using `stellar_xdr::curr::TransactionMeta`
/// 2. Match on `TransactionMeta::V3` to access `SorobanTransactionMeta.events`
/// 3. For each event, decode topics and data from `ScVal` to JSON
pub fn decode_contract_events(
    _meta_xdr_base64: &str,
    _ledger_sequence: u32,
    _tx_hash: &str,
) -> Result<Vec<ContractEvent>, IngestionError> {
    todo!("decoder::decode_contract_events: parse TransactionMeta XDR and extract Soroban events")
}
