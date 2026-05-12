# Architecture

## Problem

Building frontends for Stellar dApps requires either:
- **Horizon** — limited query flexibility, no deep Soroban event support
- **Raw XDR parsing** — verbose, error-prone, no standard query interface

This indexer sits between the Stellar network and application frontends, ingesting raw ledger data, storing it in a relational database, and serving it through a typed GraphQL API.

## Data Flow

```
Stellar Network
(Horizon REST API / Soroban RPC)
        │
        ▼
┌───────────────────┐
│  ingestion crate  │  polls /ledgers?cursor=N
│                   │  decodes XDR envelopes
│  LedgerStream     │  emits LedgerBundle per ledger
└────────┬──────────┘
         │  LedgerBundle { transactions, contract_events }
         ▼
┌───────────────────┐
│  storage crate    │  INSERT transactions
│                   │  INSERT contract_events
│  Db (Postgres)    │  tracks cursor for crash recovery
└────────┬──────────┘
         │  StorageBackend trait
         ▼
┌───────────────────┐
│  graphql crate    │  async-graphql schema
│                   │  axum HTTP server
│  POST /graphql    │  resolvers query Postgres
└───────────────────┘
```

## Crate Responsibilities

| Crate | Owns | Key types |
|-------|------|-----------|
| `ingestion` | Network I/O, XDR decoding | `LedgerStream`, `LedgerBundle`, `LedgerSource` trait |
| `storage` | Postgres schema, sqlx queries | `Db`, `StorageBackend` trait, `StoredTransaction` |
| `graphql` | Schema, resolvers, HTTP server | `QueryRoot`, `IndexerSchema`, `server::start` |
| `indexer-core` | Binary — wires all three | `main.rs` |

## Key Design Decisions

**Traits over concrete types**
`LedgerSource` and `StorageBackend` are traits. Resolvers never import `ingestion`. Tests inject an in-memory backend without a real database or network.

**XDR-first decoding**
All Stellar data is decoded from raw XDR, not from Horizon's JSON layer. This gives access to the full data model — including Soroban `TransactionMeta` events — which Horizon's JSON API does not fully expose.

**Cursor-based ingestion**
The indexer stores the last processed ledger sequence in Postgres. On restart it resumes from that point, making it crash-safe with no data loss or duplication.

**sqlx migrations**
Schema changes live in `migrations/` as plain SQL files and are applied automatically on startup via `sqlx::migrate!`. No ORM, no magic.

**Self-hostable**
The entire stack runs with `docker compose up`. No external services required beyond a Stellar Horizon endpoint (public testnet available).

## Database Schema

```sql
transactions (
    hash             TEXT PRIMARY KEY,
    ledger_sequence  INTEGER,
    source_account   TEXT,
    fee              INTEGER,
    successful       BOOLEAN,
    created_at       BIGINT
)

contract_events (
    id               BIGSERIAL PRIMARY KEY,
    contract_id      TEXT,
    ledger_sequence  INTEGER,
    tx_hash          TEXT → transactions.hash,
    topics           JSONB,
    data             JSONB
)
```

## GraphQL Schema (target)

```graphql
type Query {
  transaction(hash: String!): Transaction
  transactions(limit: Int = 20, offset: Int = 0): [Transaction!]!
  contractEvents(contractId: String!, limit: Int = 20): [ContractEvent!]!
}

type Transaction {
  hash: String!
  ledgerSequence: Int!
  sourceAccount: String!
  fee: Int!
  successful: Boolean!
}

type ContractEvent {
  contractId: String!
  ledgerSequence: Int!
  txHash: String!
  topics: [String!]!
}
```
