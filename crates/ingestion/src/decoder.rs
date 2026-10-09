//! XDR decoding utilities.

use serde_json::Value;
use stellar_xdr::curr::{
    ContractEventBody, Limits, ReadXdr, ScVal, TransactionEnvelope, TransactionMeta,
};

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

    let source_account = record["source_account"].as_str().unwrap_or("").to_string();

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

fn scval_to_json(value: &ScVal) -> Value {
    match value {
        ScVal::Bool(v) => Value::Bool(*v),
        ScVal::Void => Value::Null,
        ScVal::Error(err) => Value::String(format!("ScError::{:?}", err)),
        ScVal::U32(v) => Value::Number((*v).into()),
        ScVal::I32(v) => Value::Number((*v).into()),
        ScVal::U64(v) => Value::String(v.to_string()),
        ScVal::I64(v) => Value::String(v.to_string()),
        ScVal::Timepoint(v) => Value::String(format!("{:?}", v)),
        ScVal::Duration(v) => Value::String(format!("{:?}", v)),
        ScVal::U128(v) => Value::String(format!("{:?}", v)),
        ScVal::I128(v) => Value::String(format!("{:?}", v)),
        ScVal::U256(v) => Value::String(format!("{:?}", v)),
        ScVal::I256(v) => Value::String(format!("{:?}", v)),
        ScVal::Bytes(b) => Value::Array(b.iter().map(|byte| Value::Number((*byte).into())).collect()),
        ScVal::String(s) => Value::String(String::from_utf8_lossy(s.as_slice()).into_owned()),
        ScVal::Symbol(s) => Value::String(String::from_utf8_lossy(s.as_slice()).into_owned()),
        ScVal::Vec(items) => items
            .as_ref()
            .map(|inner| Value::Array(inner.iter().map(scval_to_json).collect()))
            .unwrap_or(Value::Array(vec![])),
        ScVal::Map(items) => items
            .as_ref()
            .map(|inner| {
                Value::Array(
                    inner.iter().map(|entry| {
                        serde_json::json!([
                            scval_to_json(&entry.key),
                            scval_to_json(&entry.val)
                        ])
                    }).collect(),
                )
            })
            .unwrap_or(Value::Array(vec![])),
        ScVal::Address(addr) => Value::String(format!("{:?}", addr)),
        ScVal::LedgerKeyContractInstance => Value::String("LedgerKeyContractInstance".to_string()),
        ScVal::LedgerKeyNonce(v) => Value::String(format!("{:?}", v)),
        ScVal::ContractInstance(v) => Value::String(format!("{:?}", v)),
    }
}

/// Decodes Soroban contract events from a `TransactionMeta` XDR string.
pub fn decode_contract_events(
    meta_xdr_base64: &str,
    ledger_sequence: u32,
    tx_hash: &str,
) -> Result<Vec<ContractEvent>, IngestionError> {
    let meta = TransactionMeta::from_xdr_base64(meta_xdr_base64, Limits::none())
        .map_err(|e: stellar_xdr::curr::Error| IngestionError::Xdr(e.to_string()))?;

    let events = match meta {
        TransactionMeta::V3(v) => v
            .soroban_meta
            .map(|meta| meta.events.iter().cloned().collect::<Vec<_>>())
            .unwrap_or_default(),
        _ => vec![],
    };

    Ok(events
        .into_iter()
        .filter_map(|event| match event.body {
            ContractEventBody::V0(body) => Some((event.contract_id, body)),
        })
        .map(|(contract_id, body)| ContractEvent {
            contract_id: contract_id.map(|id| id.to_string()).unwrap_or_default(),
            ledger_sequence,
            tx_hash: tx_hash.to_string(),
            topics: body
                .topics
                .iter()
                .map(scval_to_json)
                .map(|value| match value {
                    Value::String(s) => s,
                    _ => value.to_string(),
                })
                .collect(),
            data: scval_to_json(&body.data),
        })
        .collect())
}
