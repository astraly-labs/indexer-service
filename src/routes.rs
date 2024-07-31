use axum::http::{HeaderValue, Method, StatusCode};
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::Router;
use tower_http::cors::CorsLayer;

use crate::handlers::global::health::health_check;
use crate::handlers::indexers::create_indexer::create_indexer;
use crate::handlers::indexers::get_indexer::{
    get_indexer, get_indexer_status, get_indexer_status_by_table_name, get_indexers,
};
use crate::handlers::indexers::start_indexer::start_indexer_api;
use crate::handlers::indexers::stop_indexer::stop_indexer;
use crate::AppState;

pub fn app_router(state: AppState) -> Router<AppState> {
    Router::new()
        .nest("/", global_routes(state.clone()))
        .nest("/v1/indexers", indexers_routes(state))
        .fallback(handler_404)
        .layer(
            CorsLayer::new()
                .allow_origin("http://localhost:3000".parse::<HeaderValue>().unwrap())
                .allow_methods([Method::GET]),
        )
}

async fn handler_404() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "The requested resource was not found")
}

fn indexers_routes(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/", post(create_indexer))
        .route("/indexers", get(get_indexers))
        .route("/stop/:id", post(stop_indexer))
        .route("/start/:id", post(start_indexer_api))
        .route("/:id", get(get_indexer))
        .route("/status/:id", get(get_indexer_status))
        .route("/status/table/:table_name", get(get_indexer_status_by_table_name))
        .with_state(state)
}

fn global_routes(state: AppState) -> Router<AppState> {
    Router::new().route("/health", get(health_check)).with_state(state)
}
