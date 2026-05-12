# Architecture

## Problem

Building frontends for Stellar dApps today requires either:
- **Horizon REST API** — limited query flexibility, no filtering, no joins, incomplete Soroban event exposure
- **Raw XDR parsing** — verbose, error-prone, requires deep protocol knowledge, no standard query interface

This indexer sits between the Stellar network and application frontends. It ingests raw ledger data, normalises it into a relational database, and serves it through a typed GraphQL API that any frontend can query with full flexibility.

---

## System Overview

```
┌─────────────────────────────────────────────────────────────┐
│                      Stellar Network                        │
│         Horizon REST API  /  Soroban RPC endpoint           │
└──────────────────────────┬──────────────────────────────────┘
                           │  HTTP polling (cursor-based)
                           ▼
┌─────────────────────────────────────────────────────────────┐
│                    ingestion crate                          │
│                                                             │
│  horizon::get_ledger()         ← GET /ledgers/{seq}         │
│  horizon::get_transactions()   ← GET /ledgers/{seq}/txs     │
│  decoder::decode_transaction() ← parse TransactionEnvelope  │
│  decoder::decode_contract_events() ← parse TransactionMeta  │
│                                                             │
│  LedgerStream::next_ledger()   ← polling loop + cursor      │
│  → emits LedgerBundle { sequence, transactions, events }    │
└──────────────────────────┬──────────────────────────────────┘
                           │  LedgerBundle
                           ▼
┌─────────────────────────────────────────────────────────────┐
│                     storage crate                           │
│                                                             │
│  Db::connect()                 ← connect + run migrations   │
│  queries::insert_transaction() ← INSERT, idempotent         │
│  queries::insert_contract_event()                           │
│  queries::get_latest_ledger()  ← resume cursor on restart   │
│                                                             │
│  StorageBackend trait          ← swappable in tests         │
│  Postgres (sqlx)               ← default implementation     │
└──────────────────────────┬──────────────────────────────────┘
                           │  StorageBackend trait
                           ▼
┌─────────────────────────────────────────────────────────────┐
│                     graphql crate                           │
│                                                             │
│  QueryRoot::transaction(hash)                               │
│  QueryRoot::transactions(limit, offset)                     │
│  QueryRoot::contract_events(contract_id, limit)             │
│                                                             │
│  server::start(port)           ← axum HTTP server           │
│  POST /graphql                 ← query endpoint             │
│  GET  /graphql                 ← GraphiQL playground        │
└─────────────────────────────────────────────────────────────┘
```

---

## Crate Breakdown

### `ingestion`
Owns all network I/O and XDR decoding. Nothing outside this crate talks to Stellar directly.

| File | Responsibility |
|------|---------------|
| `src/horizon.rs` | HTTP client helpers — `get_ledger`, `get_transactions` |
| `src/decoder.rs` | XDR decoding — `decode_transaction`, `decode_contract_events` |
| `src/types.rs` | Shared output types — `LedgerBundle`, `Transaction`, `Operation`, `ContractEvent` |
| `src/lib.rs` | `LedgerSource` trait + `LedgerStream` implementation |

**Key types:**
- `LedgerSource` — trait; implement to add new data sources (archive files, Galexie, etc.)
- `LedgerStream` — polls Horizon, advances cursor, yields `LedgerBundle`
- `LedgerBundle` — all decoded data for one closed ledger

**Implementation status:**
- ✅ `horizon::get_ledger` — fetches ledger JSON from Horizon
- ✅ `horizon::get_transactions` — fetches transaction page
- ✅ `decoder::decode_transaction` — decodes Horizon JSON + `TransactionEnvelope` XDR
- 🔲 `decoder::decode_contract_events` — parse `TransactionMeta` XDR (contributor issue #1)
- 🔲 `LedgerStream::next_ledger` — polling loop (contributor issue #2)

---

### `storage`
Owns all database access. Nothing outside this crate writes SQL.

| File | Responsibility |
|------|---------------|
| `src/lib.rs` | `StorageBackend` trait + `Db` struct (Postgres impl) |
| `src/models.rs` | Row types — `StoredTransaction`, `StoredContractEvent` |
| `src/queries.rs` | Named SQL query functions |

**Key types:**
- `StorageBackend` — trait; implement for in-memory test backend or alternative DBs
- `Db` — Postgres-backed implementation using `sqlx`

**Implementation status:**
- 🔲 `Db::connect` — connect + run migrations
- 🔲 `queries::insert_transaction`
- 🔲 `queries::insert_contract_event`
- 🔲 `queries::get_latest_ledger`

---

### `graphql`
Owns the HTTP server and GraphQL schema. Depends only on `storage` — never imports `ingestion`.

| File | Responsibility |
|------|---------------|
| `src/lib.rs` | Public re-exports |
| `src/schema.rs` | `QueryRoot`, GraphQL types, `build_schema()` |
| `src/server.rs` | `server::start(port)` — axum HTTP server |

**Implementation status:**
- 🔲 `QueryRoot::transaction`
- 🔲 `QueryRoot::transactions`
- 🔲 `QueryRoot::contract_events`
- 🔲 `server::start`

---

### `indexer-core`
Binary entry point. Loads config from env, wires the three crates together, handles shutdown.

| File | Responsibility |
|------|---------------|
| `src/main.rs` | Startup — config, DB connect, spawn ingestion loop, start GraphQL server |

**Implementation status:**
- 🔲 Config loading from env
- 🔲 Wire ingestion → storage pipeline
- 🔲 Start GraphQL server

---

## Database Schema

```sql
-- migrations/0001_initial.sql

CREATE TABLE transactions (
    hash             TEXT        PRIMARY KEY,
    ledger_sequence  INTEGER     NOT NULL,
    source_account   TEXT        NOT NULL,
    fee              INTEGER     NOT NULL,
    successful       BOOLEAN     NOT NULL,
    created_at       BIGINT      NOT NULL
);

CREATE INDEX idx_transactions_ledger ON transactions (ledger_sequence);
CREATE INDEX idx_transactions_source ON transactions (source_account);

CREATE TABLE contract_events (
    id               BIGSERIAL   PRIMARY KEY,
    contract_id      TEXT        NOT NULL,
    ledger_sequence  INTEGER     NOT NULL,
    tx_hash          TEXT        NOT NULL REFERENCES transactions(hash),
    topics           JSONB       NOT NULL DEFAULT '[]',
    data             JSONB       NOT NULL DEFAULT '{}'
);

CREATE INDEX idx_events_contract ON contract_events (contract_id);
CREATE INDEX idx_events_ledger   ON contract_events (ledger_sequence);
```

---

## GraphQL API

### Current schema (target)

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

### Endpoints

| Method | Path | Description |
|--------|------|-------------|
| `POST` | `/graphql` | Query endpoint |
| `GET` | `/graphql` | GraphiQL playground (dev) |

---

## Key Design Decisions

**Traits over concrete types**
`LedgerSource` and `StorageBackend` are traits. The GraphQL resolvers depend only on `StorageBackend` — they never import `ingestion`. Tests inject an in-memory backend with no database or network required.

**XDR-first decoding**
All Stellar data is decoded from raw XDR (`stellar-xdr` crate), not from Horizon's JSON layer. This gives access to the full protocol data model — including Soroban `TransactionMeta` events — which Horizon's JSON API does not fully expose.

**Cursor-based ingestion**
The last processed ledger sequence is stored in Postgres. On restart the indexer resumes from that point — no data loss, no duplication, crash-safe by design.

**sqlx migrations**
Schema changes are plain SQL files in `migrations/`. Applied automatically on startup via `sqlx::migrate!`. No ORM, no code generation required.

**Self-hostable**
The full stack runs with `docker compose up`. The only external dependency is a Stellar Horizon endpoint — the public testnet (`https://horizon-testnet.stellar.org`) works out of the box.

---

## Configuration

All config is loaded from environment variables. See `.env.example` for defaults.

| Variable | Description | Default |
|----------|-------------|---------|
| `HORIZON_URL` | Stellar Horizon endpoint | `https://horizon-testnet.stellar.org` |
| `SOROBAN_RPC_URL` | Soroban RPC endpoint (optional) | `https://soroban-testnet.stellar.org` |
| `DATABASE_URL` | Postgres connection string | `postgres://postgres:postgres@localhost:5432/stellar_indexer` |
| `START_LEDGER` | Ledger sequence to start from | `latest` |
| `GRAPHQL_PORT` | Port for the GraphQL server | `4000` |
| `LOG_LEVEL` | Tracing log level | `info` |

---

## Running Locally

```bash
cp .env.example .env
docker compose up -d postgres   # start Postgres
cargo run -p indexer-core       # start the indexer
# GraphQL available at http://localhost:4000/graphql
```

## Running Tests

```bash
cargo test --all
```

Storage tests use an in-memory `StorageBackend` — no database required.
Ingestion tests that hit Horizon are marked `#[ignore]` and run with:
```bash
cargo test --all -- --include-ignored
```
