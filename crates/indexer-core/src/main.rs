//! stellar-graphql-indexer — entry point.
//!
//! Wires together the ingestion pipeline, storage layer, and GraphQL server.
//! Configuration is loaded from environment variables (see `.env.example`).

use std::sync::Arc;

use ingestion::LedgerSource;
use storage::{
    models::{StoredContractEvent, StoredTransaction},
    StorageBackend,
};
use tracing::info;

#[derive(Clone, Debug)]
struct Config {
    horizon_url: String,
    database_url: String,
    graphql_port: u16,
    start_ledger: u32,
}

fn load_config() -> Config {
    let _ = dotenvy::dotenv();

    let horizon_url = std::env::var("HORIZON_URL").unwrap_or_else(|_| "https://horizon-testnet.stellar.org".to_string());
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://postgres:postgres@localhost:5432/stellar_indexer".to_string());
    let graphql_port = std::env::var("GRAPHQL_PORT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(4000);
    let start_ledger = std::env::var("START_LEDGER")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);

    Config {
        horizon_url,
        database_url,
        graphql_port,
        start_ledger,
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = load_config();

    tracing_subscriber::fmt()
        .with_env_filter(std::env::var("LOG_LEVEL").unwrap_or_else(|_| "info".into()))
        .init();

    info!("stellar-graphql-indexer starting");

    let db = storage::Db::connect(&config.database_url).await?;
    let storage: Arc<dyn StorageBackend> = Arc::new(db);

    let current_ledger = storage.get_latest_ledger().await.unwrap_or(0);
    let start_ledger = config.start_ledger.max(current_ledger);
    let mut stream = ingestion::LedgerStream::new(&config.horizon_url, start_ledger);
    let ingestion_storage = storage.clone();

    tokio::spawn(async move {
        loop {
            match stream.next_ledger().await {
                Ok(bundle) => {
                    info!(ledger = bundle.sequence, tx_count = bundle.transactions.len(), event_count = bundle.contract_events.len(), "ingested ledger");

                    for tx in bundle.transactions {
                        let stored = StoredTransaction {
                            hash: tx.hash,
                            ledger_sequence: tx.ledger_sequence as i32,
                            source_account: tx.source_account,
                            fee: tx.fee as i32,
                            successful: tx.successful,
                            created_at: std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .unwrap_or_default()
                                .as_millis() as i64,
                        };
                        if let Err(err) = ingestion_storage.insert_transaction(&stored).await {
                            tracing::error!(error = %err, "failed to insert transaction");
                        }
                    }

                    for event in bundle.contract_events {
                        let stored = StoredContractEvent {
                            id: 0,
                            contract_id: event.contract_id,
                            ledger_sequence: event.ledger_sequence as i32,
                            tx_hash: event.tx_hash,
                            topics: serde_json::json!(event.topics),
                            data: event.data,
                        };
                        if let Err(err) = ingestion_storage.insert_contract_event(&stored).await {
                            tracing::error!(error = %err, "failed to insert contract event");
                        }
                    }
                }
                Err(err) => {
                    tracing::error!(error = %err, "ledger ingestion failed");
                    tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                }
            }
        }
    });

    graphql::server::start_with_storage(config.graphql_port, storage).await?;
    info!("all components wired — ready");
    tokio::signal::ctrl_c().await?;
    info!("shutting down");
    Ok(())
}
