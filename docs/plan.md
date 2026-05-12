# Build Plan

Tasks are ordered from the most foundational (blocking everything else) to the least critical (polish and extras). Each phase ends with a working, testable milestone.

---

## Phase 1 — Ingestion (Core)
*Nothing else works without data coming in.*

| # | Task | File | Notes |
|---|------|------|-------|
| 1.1 | `horizon::get_ledger` — fetch single ledger JSON | `ingestion/src/horizon.rs` | GET `/ledgers/{seq}` |
| 1.2 | `horizon::get_transactions` — fetch tx page | `ingestion/src/horizon.rs` | GET `/ledgers/{seq}/transactions` |
| 1.3 | `decoder::decode_transaction` — parse `TransactionEnvelope` XDR | `ingestion/src/decoder.rs` | Use `stellar-xdr` crate |
| 1.4 | `decoder::decode_contract_events` — extract Soroban events from `TransactionMeta` | `ingestion/src/decoder.rs` | Key for Soroban support |
| 1.5 | `LedgerStream::next_ledger` — polling loop with cursor | `ingestion/src/lib.rs` | Ties 1.1–1.4 together |

**Milestone:** `cargo run` connects to testnet and logs decoded ledger data to stdout.

---

## Phase 2 — Storage
*Data must be persisted before it can be queried.*

| # | Task | File | Notes |
|---|------|------|-------|
| 2.1 | `queries::insert_transaction` — sqlx INSERT | `storage/src/queries.rs` | Idempotent on hash conflict |
| 2.2 | `queries::insert_contract_event` — sqlx INSERT | `storage/src/queries.rs` | |
| 2.3 | `queries::get_latest_ledger` — resume cursor | `storage/src/queries.rs` | Crash recovery |
| 2.4 | `Db::connect` — connect + run migrations | `storage/src/lib.rs` | `sqlx::migrate!` |
| 2.5 | Wire ingestion → storage in `main.rs` | `indexer-core/src/main.rs` | End-to-end pipeline |

**Milestone:** Indexer runs, ingests 10 ledgers, rows visible in Postgres.

---

## Phase 3 — GraphQL API
*The user-facing layer. Depends on Phase 2.*

| # | Task | File | Notes |
|---|------|------|-------|
| 3.1 | `QueryRoot::transaction(hash)` resolver | `graphql/src/schema.rs` | Single lookup |
| 3.2 | `QueryRoot::transactions(limit, offset)` resolver | `graphql/src/schema.rs` | Paginated list |
| 3.3 | `QueryRoot::contract_events(contract_id, limit)` resolver | `graphql/src/schema.rs` | Filter by contract |
| 3.4 | `server::start(port)` — axum server | `graphql/src/server.rs` | GraphiQL playground included |
| 3.5 | Wire schema + storage in `main.rs` | `indexer-core/src/main.rs` | Inject storage as context |

**Milestone:** `curl -X POST localhost:4000/graphql -d '{"query":"{transactions{hash}}"}'` returns real data.

---

## Phase 4 — Extended Queries
*Useful but not blocking the core use case.*

| # | Task | Notes |
|---|------|-------|
| 4.1 | `account(id)` query — balance + tx history | Requires account balance tracking in storage |
| 4.2 | `ledger(sequence)` query — metadata, tx count | Simple SELECT |
| 4.3 | Keyset pagination (cursor-based) | Replace offset with `after: String` cursor |
| 4.4 | Filter transactions by account | Add index on `source_account` |

---

## Phase 5 — Operations & Polish
*Nice to have. Contributor-friendly issues.*

| # | Task | Notes |
|---|------|-------|
| 5.1 | Prometheus `/metrics` endpoint | Expose ingestion lag, tx count, error rate |
| 5.2 | GraphQL subscriptions for live events | `async-graphql` subscription support |
| 5.3 | Rate limiting on GraphQL endpoint | `tower` middleware |
| 5.4 | Structured error responses | Map `StorageError` → GraphQL errors cleanly |
| 5.5 | Docker image published to GHCR | GitHub Actions release workflow |

---

## Contributor Issue Map

Each `todo!()` in the codebase maps to a future GitHub issue. Phases 1–3 are maintainer-led. Phases 4–5 are opened as contributor issues once Phase 3 is complete.

```
Phase 1 → maintainer implements (establishes the project)
Phase 2 → maintainer implements (establishes the project)
Phase 3 → maintainer implements (establishes the project)
Phase 4 → open as complexity: medium issues
Phase 5 → open as complexity: trivial / medium issues
```
