# Contributing to stellar-graphql-indexer

Thanks for your interest! This document covers everything you need to get started.

## Prerequisites

- [Rust](https://rustup.rs/) stable (1.74+)
- [Docker](https://docs.docker.com/get-docker/) + Docker Compose (for Postgres)
- A Stellar Horizon endpoint (public testnet: `https://horizon-testnet.stellar.org`)

## Getting Started

```bash
git clone https://github.com/Soro-Testing-ORG/Stellar-GraphQL-Indexer
cd Stellar-GraphQL-Indexer
cp .env.example .env
docker compose up -d postgres
cargo build
cargo test
```

## Project Layout

| Crate | Purpose |
|-------|---------|
| `crates/indexer-core` | Binary — wires ingestion → storage → graphql |
| `crates/ingestion` | Streams ledger data from Horizon/RPC, decodes XDR |
| `crates/storage` | Postgres persistence via sqlx |
| `crates/graphql` | GraphQL schema and resolvers via async-graphql |

## How to Contribute

1. **Find an issue** — issues are labeled by complexity:
   - `complexity: trivial` — docs, small fixes
   - `complexity: medium` — standard feature or bug fix
   - `complexity: high` — complex feature or refactor

2. **Comment on the issue** before starting work.

3. **Fork and branch** — e.g. `feat/account-resolver` or `fix/ledger-decode`.

4. **Write tests** — all new logic needs tests. Run `cargo test` before opening a PR.

5. **Open a PR** — link the issue with `Closes #<number>`.

## Code Style

```bash
cargo fmt --all
cargo clippy --all-targets -- -D warnings
```

Both must pass before a PR is merged.

## Questions?

Open a [GitHub Discussion](https://github.com/Soro-Testing-ORG/Stellar-GraphQL-Indexer/discussions).
