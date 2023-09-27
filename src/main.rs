use std::net::SocketAddr;
use std::sync::Arc;

use deadpool_diesel::postgres::Pool;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};

use crate::config::config;
use crate::consumers::init_consumers;
use crate::errors::internal_error;
use crate::handlers::indexers::start_indexer::start_all_indexers;
use crate::routes::app_router;

/// Configuration of the service (AWS, DB, etc)
mod config;
/// Constants used accross the service
pub mod constants;
/// SQS consumers
mod consumers;
/// Database domain models
mod domain;
/// Error handling
mod errors;
/// Route endpoints handlers
mod handlers;
/// Database utils (repositories, error handling, etc)
mod infra;
/// SQS message publishers
pub mod publishers;
/// Route endpoints definitions
mod routes;
/// Utilities
mod utils;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations/");

#[derive(Clone)]
pub struct AppState {
    pool: Arc<Pool>,
}

#[tokio::main]
async fn main() {
    init_tracing();

    let config = config().await;

    run_migrations(config.pool()).await;

    let state = AppState { pool: Arc::clone(config.pool()) };

    let app = app_router(state.clone()).with_state(state);

    let host = config.server_host();
    let port = config.server_port();

    let address = format!("{}:{}", host, port);

    let socket_addr: SocketAddr = address.parse().expect("Failed to parse socket address");

    tracing::info!("listening on http://{}", socket_addr);

    // initializes the SQS messages consumers
    init_consumers();

    // start all indexers that were running before the service was stopped
    start_all_indexers().await.expect("Failed to start all the indexers");

    axum::Server::bind(&socket_addr)
        .serve(app.into_make_service())
        .await
        .map_err(internal_error)
        .expect("Failed to start the server");
}

fn init_tracing() {
    tracing_subscriber::fmt().with_max_level(tracing::Level::INFO).with_target(false).init();
}

async fn run_migrations(pool: &Pool) {
    let conn = pool.get().await.expect("Failed to get a connection from the pool");
    conn.interact(|conn| conn.run_pending_migrations(MIGRATIONS).map(|_| ()))
        .await
        .expect("Failed to run pending migrations")
        .expect("Failed to interact with the database");
}
