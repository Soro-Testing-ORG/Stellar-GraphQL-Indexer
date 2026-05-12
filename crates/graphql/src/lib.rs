//! GraphQL layer — schema, resolvers, and HTTP server.
//!
//! Built on [`async-graphql`] and served via [`axum`].
//! The schema is split into query types that mirror the storage models.

pub mod schema;
pub mod server;

pub use server::start;
