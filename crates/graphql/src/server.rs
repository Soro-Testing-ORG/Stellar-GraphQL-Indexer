//! Axum HTTP server that serves the GraphQL endpoint.
//!
//! Exposes:
//!   POST /graphql  — query endpoint
//!   GET  /graphql  — GraphiQL playground (dev only)

use std::sync::Arc;

use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::{
    extract::Extension,
    response::{Html, IntoResponse},
    routing::{get, post},
    Router,
};
use storage::StorageBackend;

use crate::schema::{build_schema_with_storage, IndexerSchema};

pub async fn start(port: u16) -> anyhow::Result<()> {
    let noop: Arc<dyn StorageBackend> = Arc::new(NoopStorage);
    start_with_storage(port, noop).await
}

pub async fn start_with_storage(
    port: u16,
    storage: Arc<dyn StorageBackend>,
) -> anyhow::Result<()> {
    let schema = build_schema_with_storage(storage);
    let app = Router::new()
        .route("/graphql", post(graphql_handler))
        .route("/graphql", get(graphql_playground))
        .layer(Extension(schema));

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}")).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

#[allow(dead_code)]
async fn graphql_handler(
    schema: Extension<IndexerSchema>,
    req: GraphQLRequest,
) -> GraphQLResponse {
    schema.execute(req.into_inner()).await.into()
}

async fn graphql_playground() -> impl IntoResponse {
    Html(async_graphql::http::playground_source(
        async_graphql::http::GraphQLPlaygroundConfig::new("/graphql"),
    ))
}

struct NoopStorage;

#[async_trait::async_trait]
impl StorageBackend for NoopStorage {
    async fn insert_transaction(&self, _tx: &storage::models::StoredTransaction) -> Result<(), storage::StorageError> {
        Ok(())
    }

    async fn insert_contract_event(&self, _event: &storage::models::StoredContractEvent) -> Result<(), storage::StorageError> {
        Ok(())
    }

    async fn get_transaction(&self, _hash: &str) -> Result<storage::models::StoredTransaction, storage::StorageError> {
        Err(storage::StorageError::NotFound)
    }

    async fn get_latest_ledger(&self) -> Result<u32, storage::StorageError> {
        Ok(0)
    }
}
