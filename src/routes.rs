use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::Router;

use crate::handlers::global::health::health_check;
use crate::handlers::indexers::create_indexer::create_indexer;
use crate::handlers::indexers::get_indexer::get_indexer;
use crate::handlers::indexers::start_indexer::start_indexer_api;
use crate::handlers::indexers::stop_indexer::stop_indexer;
use crate::AppState;

pub fn app_router(state: AppState) -> Router<AppState> {
    Router::new()
        .nest("/", global_routes(state.clone()))
        .nest("/v1/indexers", indexers_routes(state))
        .fallback(handler_404)
}

async fn handler_404() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "The requested resource was not found")
}

fn indexers_routes(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/", post(create_indexer))
        .route("/stop/:id", post(stop_indexer))
        .route("/start/:id", post(start_indexer_api))
        .route("/:id", get(get_indexer))
        .with_state(state)
}

fn global_routes(state: AppState) -> Router<AppState> {
    Router::new().route("/health", get(health_check)).with_state(state)
}
