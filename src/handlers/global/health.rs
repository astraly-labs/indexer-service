use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;

use crate::AppState;

pub async fn health_check(State(state): State<AppState>) -> impl IntoResponse {
    StatusCode::OK
}
