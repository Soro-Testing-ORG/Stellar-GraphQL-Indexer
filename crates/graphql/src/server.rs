//! Axum HTTP server that serves the GraphQL endpoint.
//!
//! Exposes:
//!   POST /graphql  — query endpoint
//!   GET  /graphql  — GraphiQL playground (dev only)

use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::{routing::get, Router};

use crate::schema::{build_schema, IndexerSchema};

/// Starts the GraphQL HTTP server on the given port.
///
/// # TODO
/// 1. Build the schema with a real storage backend injected as context
/// 2. Mount `/graphql` POST handler and GraphiQL GET handler
/// 3. Bind and serve with `axum::serve`
pub async fn start(port: u16) -> anyhow::Result<()> {
    let _ = port;
    todo!("server::start: bind axum router and serve")
}

async fn graphql_handler(
    schema: axum::extract::Extension<IndexerSchema>,
    req: GraphQLRequest,
) -> GraphQLResponse {
    schema.execute(req.into_inner()).await.into()
}
