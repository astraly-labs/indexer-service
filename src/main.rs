use std::net::SocketAddr;

use deadpool_diesel::postgres::Pool;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};

use crate::config::config;
use crate::consumers::init_consumers;
use crate::errors::internal_error;
use crate::handlers::indexers::start_indexer::start_all_indexers;
use crate::routes::app_router;

mod config;
pub mod constants;
mod consumers;
mod domain;
mod errors;
mod handlers;
mod infra;
pub mod publishers;
mod routes;
mod utils;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations/");

#[derive(Clone)]
pub struct AppState {
    pool: Pool,
}

#[tokio::main]
async fn main() {
    init_tracing();

    let config = config().await;

    run_migrations(config.pool()).await;

    // TODO: is it safe to clone this?
    let state = AppState { pool: config.pool().clone() };

    let app = app_router(state.clone()).with_state(state);

    let host = config.server_host();
    let port = config.server_port();

    let address = format!("{}:{}", host, port);

    let socket_addr: SocketAddr = address.parse().unwrap();

    tracing::info!("listening on http://{}", socket_addr);
    init_consumers();

    // start all indexers that were running before the service was stopped
    start_all_indexers().await.expect("Failed to start all the indexers");
    axum::Server::bind(&socket_addr).serve(app.into_make_service()).await.map_err(internal_error).unwrap()
}

fn init_tracing() {
    tracing_subscriber::fmt().with_max_level(tracing::Level::INFO).with_target(false).init();
}

async fn run_migrations(pool: &Pool) {
    let conn = pool.get().await.unwrap();
    conn.interact(|conn| conn.run_pending_migrations(MIGRATIONS).map(|_| ())).await.unwrap().unwrap();
}
