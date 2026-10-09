# stellar-graphql-indexer

A self-hostable indexer that exposes Stellar and Soroban data via a GraphQL API.

Building frontends for Stellar dApps today means either using Horizon (limited query flexibility) or writing raw XDR parsers from scratch. This indexer bridges that gap: it ingests ledger data from the Stellar network, stores it in a queryable database, and serves it through a typed GraphQL schema.

## Features

- **Ledger ingestion** — streams ledger closes from Stellar Core / Horizon and decodes XDR
- **Soroban support** — decodes and indexes contract events from transaction metadata
- **GraphQL API** — query transactions and contract events with bounded pagination
- **Self-hostable** — runs with Docker Compose; bring your own Postgres
- **Pluggable storage** — storage layer is behind a trait; Postgres ships by default

## Recent improvements

- **Optimized ingestion pipeline** — the ingestion loop now advances via ledger cursor, decodes each transaction bundle, and persists rows without reprocessing the latest seen ledger.
- **Expanded schema coverage** — Soroban event decoding extracts contract IDs, topics, and payload data from `TransactionMeta` XDR, and the GraphQL layer is wired to a shared storage backend.
- **Resilience and recovery** — failed ledger fetches/writes are retried after a delay, and the indexer resumes after the highest atomically checkpointed ledger on startup.
- **Deployment simplicity** — environment variables are loaded from `.env`/`.env.example` and Docker Compose is configured for local Postgres and the indexer service.

## Status

🚧 **Early development.** Core ingestion, checkpointed storage, and transaction/event queries are implemented, but production operations and broader Soroban coverage remain in progress. Contributions welcome — see [CONTRIBUTING.md](CONTRIBUTING.md).

## Quick Start

For local development with the indexer running on your host:

```bash
cp .env.example .env
docker compose up -d postgres
cargo run -p indexer-core
```

To run both services in containers instead, use `docker compose up -d`. The Compose
configuration connects the indexer container to Postgres by service name; set
`DATABASE_URL_DOCKER` in `.env` only when overriding that connection. Query at
`http://localhost:4000/graphql`.

## Project Structure

```
crates/
  indexer-core/     # Binary entry point — wires all crates together
  ingestion/        # Connects to Stellar, streams ledgers, decodes XDR
  storage/          # Postgres persistence layer (sqlx)
  graphql/          # async-graphql schema, resolvers, server
migrations/         # sqlx database migrations
docs/
  architecture.md   # Design decisions and data flow
```

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md). Issues are labeled by complexity.

## License

MIT
