//! stellar-graphql-indexer — entry point.
//!
//! Wires together the ingestion pipeline, storage layer, and GraphQL server.

use std::{str::FromStr, sync::Arc, time::Duration};

use ingestion::LedgerSource;
use storage::{
    models::{StoredContractEvent, StoredTransaction},
    StorageBackend,
};
use tracing::info;
use tracing_subscriber::EnvFilter;

#[derive(Clone, Debug)]
struct Config {
    horizon_url: String,
    database_url: String,
    graphql_port: u16,
    start_ledger: Option<u32>,
}

fn load_config() -> anyhow::Result<Config> {
    match dotenvy::dotenv() {
        Ok(_) => {}
        Err(dotenvy::Error::Io(err)) if err.kind() == std::io::ErrorKind::NotFound => {}
        Err(err) => return Err(err.into()),
    }

    let graphql_port = std::env::var("GRAPHQL_PORT")
        .unwrap_or_else(|_| "4000".to_string())
        .parse::<u16>()?;
    let start_ledger = std::env::var("START_LEDGER")
        .ok()
        .map(|value| value.parse::<u32>())
        .transpose()?;

    Ok(Config {
        horizon_url: std::env::var("HORIZON_URL")
            .unwrap_or_else(|_| "https://horizon-testnet.stellar.org".to_string()),
        database_url: std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgresql://postgres:postgres@localhost:5432/stellar_indexer".to_string()
        }),
        graphql_port,
        start_ledger,
    })
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = load_config()?;
    let filter = std::env::var("LOG_LEVEL").unwrap_or_else(|_| "info".to_string());
    let filter = EnvFilter::from_str(&filter)?;
    tracing_subscriber::fmt().with_env_filter(filter).init();

    info!("stellar-graphql-indexer starting");

    let db = storage::Db::connect(&config.database_url).await?;
    let storage: Arc<dyn StorageBackend> = Arc::new(db);
    let latest_ledger = storage.get_latest_ledger().await?;
    let resume_ledger = latest_ledger
        .checked_add(1)
        .ok_or_else(|| anyhow::anyhow!("ledger sequence overflow"))?;
    let start_ledger = config
        .start_ledger
        .unwrap_or(if latest_ledger == 0 { 1 } else { resume_ledger })
        .max(resume_ledger)
        .max(1);

    info!(start_ledger, "starting ledger ingestion");
    let mut stream = ingestion::LedgerStream::new(&config.horizon_url, start_ledger);
    let ingestion_storage = Arc::clone(&storage);

    let ingestion_task = tokio::spawn(async move {
        loop {
            match stream.next_ledger().await {
                Ok(bundle) => {
                    let transactions = bundle
                        .transactions
                        .into_iter()
                        .map(|tx| StoredTransaction {
                            hash: tx.hash,
                            ledger_sequence: tx.ledger_sequence as i32,
                            source_account: tx.source_account,
                            fee: tx.fee as i32,
                            successful: tx.successful,
                            created_at: bundle.closed_at,
                        })
                        .collect::<Vec<_>>();
                    let events = bundle
                        .contract_events
                        .into_iter()
                        .map(|event| StoredContractEvent {
                            id: 0,
                            event_index: event.event_index as i32,
                            contract_id: event.contract_id,
                            ledger_sequence: event.ledger_sequence as i32,
                            tx_hash: event.tx_hash,
                            topics: serde_json::json!(event.topics),
                            data: event.data,
                        })
                        .collect::<Vec<_>>();

                    match ingestion_storage
                        .persist_ledger(bundle.sequence, bundle.closed_at, &transactions, &events)
                        .await
                    {
                        Ok(()) => info!(
                            ledger = bundle.sequence,
                            tx_count = transactions.len(),
                            event_count = events.len(),
                            "persisted ledger"
                        ),
                        Err(err) => {
                            stream.cursor = bundle.sequence;
                            tracing::error!(
                                ledger = bundle.sequence,
                                error = %err,
                                "failed to persist ledger; will retry"
                            );
                            tokio::time::sleep(Duration::from_secs(5)).await;
                        }
                    }
                }
                Err(err) => {
                    tracing::error!(ledger = stream.cursor, error = %err, "ledger ingestion failed");
                    tokio::time::sleep(Duration::from_secs(5)).await;
                }
            }
        }
    });

    let server_result =
        graphql::server::start_with_storage(config.graphql_port, Arc::clone(&storage)).await;
    ingestion_task.abort();
    server_result?;
    info!("shutting down");
    Ok(())
}
