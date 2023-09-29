use std::net::SocketAddr;
use std::sync::Arc;

use diesel::ConnectionError;
use diesel_async::async_connection_wrapper::AsyncConnectionWrapper;
use diesel_async::pooled_connection::deadpool::Pool;
use diesel_async::AsyncPgConnection;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use errors::AppError;

use crate::config::{config, establish_connection};
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
    pool: Arc<Pool<AsyncPgConnection>>,
}

#[tokio::main]
async fn main() -> Result<(), AppError> {
    init_tracing();

    let config = config().await;

    run_migrations(config.db_url().to_string()).await.map_err(AppError::DbError)?;

    let state = AppState { pool: Arc::clone(config.pool()) };

    let app = app_router(state.clone()).with_state(state);

    let host = config.server_host();
    let port = config.server_port();

    let address = format!("{}:{}", host, port);

    let socket_addr: SocketAddr = address.parse().expect("Failed to parse socket address");

    tracing::info!("listening on http://{}", socket_addr);

    // initializes the SQS messages consumers
    init_consumers().await.map_err(AppError::Indexer)?;

    // start all indexers that were running before the service was stopped
    start_all_indexers().await.map_err(AppError::Indexer)?;

    axum::Server::bind(&socket_addr).serve(app.into_make_service()).await.map_err(internal_error)?;

    Ok(())
}

fn init_tracing() {
    tracing_subscriber::fmt().with_max_level(tracing::Level::INFO).with_target(false).init();
}

async fn run_migrations(db_url: String) -> Result<(), ConnectionError> {
    let async_connection = establish_connection(db_url.as_str()).await?;
    let mut async_wrapper: AsyncConnectionWrapper<AsyncPgConnection> = AsyncConnectionWrapper::from(async_connection);
    let _ = tokio::task::spawn_blocking(move || {
        async_wrapper.run_pending_migrations(MIGRATIONS).unwrap();
    })
    .await;
    Ok(())
}
