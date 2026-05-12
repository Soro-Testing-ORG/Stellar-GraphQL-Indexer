//! XDR decoding utilities.
//!
//! Converts raw Stellar XDR (base64-encoded strings from Horizon) into
//! the typed structs defined in [`crate::types`].

use crate::types::{ContractEvent, Operation, Transaction};
use crate::IngestionError;

/// Decodes a base64-encoded `TransactionEnvelope` XDR string into a [`Transaction`].
///
/// # TODO
/// 1. Base64-decode the input string
/// 2. Parse with `stellar_xdr::TransactionEnvelope::from_xdr_base64`
/// 3. Map fields into `Transaction`
pub fn decode_transaction(
    _xdr_base64: &str,
    _ledger_sequence: u32,
    _successful: bool,
) -> Result<Transaction, IngestionError> {
    todo!("decoder::decode_transaction: parse TransactionEnvelope XDR")
}

/// Decodes Soroban contract events from a `TransactionMeta` XDR string.
///
/// # TODO
/// 1. Parse `TransactionMeta` XDR
/// 2. Extract `SorobanTransactionMeta.events`
/// 3. Decode each event's topics and data from `ScVal` XDR
pub fn decode_contract_events(
    _meta_xdr_base64: &str,
    _ledger_sequence: u32,
    _tx_hash: &str,
) -> Result<Vec<ContractEvent>, IngestionError> {
    todo!("decoder::decode_contract_events: parse TransactionMeta XDR and extract events")
}

/// Decodes a single `Operation` body from XDR into a typed [`Operation`].
///
/// # TODO
/// Match on `stellar_xdr::OperationBody` variants and serialize to JSON.
pub fn decode_operation(_op_xdr: &stellar_xdr::curr::Operation) -> Operation {
    todo!("decoder::decode_operation: match OperationBody variants")
}
