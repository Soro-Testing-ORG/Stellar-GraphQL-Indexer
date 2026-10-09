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

pub async fn start_with_storage(port: u16, storage: Arc<dyn StorageBackend>) -> anyhow::Result<()> {
    let schema = build_schema_with_storage(storage);
    let app = Router::new()
        .route("/graphql", post(graphql_handler))
        .route("/graphql", get(graphql_playground))
        .layer(Extension(schema));

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}")).await?;
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    Ok(())
}

async fn shutdown_signal() {
    #[cfg(unix)]
    {
        match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()) {
            Ok(mut terminate) => {
                tokio::select! {
                    result = tokio::signal::ctrl_c() => {
                        if let Err(err) = result {
                            tracing::error!(error = %err, "failed waiting for interrupt signal");
                        }
                    }
                    _ = terminate.recv() => {}
                }
            }
            Err(err) => {
                tracing::error!(error = %err, "failed to install termination signal handler");
                if let Err(err) = tokio::signal::ctrl_c().await {
                    tracing::error!(error = %err, "failed waiting for interrupt signal");
                }
            }
        }
    }

    #[cfg(not(unix))]
    if let Err(err) = tokio::signal::ctrl_c().await {
        tracing::error!(error = %err, "failed waiting for interrupt signal");
    }
}

#[allow(dead_code)]
async fn graphql_handler(schema: Extension<IndexerSchema>, req: GraphQLRequest) -> GraphQLResponse {
    schema.execute(req.into_inner()).await.into()
}

async fn graphql_playground() -> impl IntoResponse {
    Html(async_graphql::http::playground_source(
        async_graphql::http::GraphQLPlaygroundConfig::new("/graphql"),
    ))
}
