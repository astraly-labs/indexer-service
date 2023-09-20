use axum::handler::Handler;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::Router;

use crate::handlers::indexers::create_indexer::create_indexer;
use crate::handlers::indexers::get_indexer::get_indexer;
use crate::handlers::indexers::stop_indexer::stop_indexer;
use crate::AppState;

pub fn app_router(state: AppState) -> Router<AppState> {
    Router::new().route("/", get(root)).nest("/v1/indexers", indexers_routes(state.clone())).fallback(handler_404)
}

async fn root() -> &'static str {
    "Server is running!"
}

async fn handler_404() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "The requested resource was not found")
}

fn indexers_routes(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/", post(create_indexer))
        .route("/stop/:id", get(stop_indexer))
        .route("/:id", get(get_indexer))
        .with_state(state)
}
