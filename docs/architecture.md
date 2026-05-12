# Architecture

## Data Flow

```
Stellar Network (Horizon / Soroban RPC)
        │
        ▼
  ingestion::LedgerStream
  - polls /ledgers?cursor=N
  - decodes XDR envelopes
  - emits LedgerBundle
        │
        ▼
  storage::Db (Postgres)
  - transactions table
  - contract_events table
  - indexed by ledger_sequence, account, contract_id
        │
        ▼
  graphql::Server (axum + async-graphql)
  - POST /graphql  — query endpoint
  - GET  /graphql  — GraphiQL playground
```

## Crate Responsibilities

| Crate | Owns |
|-------|------|
| `ingestion` | Network I/O, XDR decoding, `LedgerSource` trait |
| `storage` | Postgres schema, sqlx queries, `StorageBackend` trait |
| `graphql` | async-graphql schema, resolvers, axum server |
| `indexer-core` | Binary — wires the three crates, loads config |

## Key Design Decisions

**Traits over concrete types** — `LedgerSource` and `StorageBackend` are traits. This means the GraphQL resolvers never import `ingestion`, and tests can inject an in-memory backend without a real database.

**XDR first** — all Stellar data is decoded from XDR, not from Horizon's JSON API. This gives us access to the full data model including Soroban contract events, which Horizon's JSON layer doesn't fully expose.

**Cursor-based ingestion** — the indexer tracks the last processed ledger sequence in Postgres. On restart it resumes from where it left off, making it crash-safe.

**sqlx migrations** — schema changes live in `migrations/` and are applied automatically on startup via `sqlx::migrate!`.
