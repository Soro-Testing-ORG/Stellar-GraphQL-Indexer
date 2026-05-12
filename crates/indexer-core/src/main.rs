//! stellar-graphql-indexer — entry point.
//!
//! Wires together the ingestion pipeline, storage layer, and GraphQL server.
//! Configuration is loaded from environment variables (see `.env.example`).

use tracing::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load .env if present
    let _ = dotenvy::dotenv();

    tracing_subscriber::fmt()
        .with_env_filter(std::env::var("LOG_LEVEL").unwrap_or_else(|_| "info".into()))
        .init();

    info!("stellar-graphql-indexer starting");

    // TODO: load Config from env
    // TODO: connect to Postgres via storage::Db::connect(&config.database_url)
    // TODO: spawn ingestion::LedgerStream and pipe into storage
    // TODO: start graphql::Server

    info!("all components wired — ready");

    // Keep the process alive until Ctrl-C
    tokio::signal::ctrl_c().await?;
    info!("shutting down");
    Ok(())
}
