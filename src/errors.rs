use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde_json::json;

use crate::domain::models::indexer::IndexerError;

#[derive(Debug)]
pub enum AppError {
    InternalServer,
    BodyParsing(String),
    Indexer(IndexerError),
}

pub fn internal_error<E>(_err: E) -> AppError {
    AppError::InternalServer
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let (status, err_msg) = match self {
            Self::InternalServer => (StatusCode::INTERNAL_SERVER_ERROR, String::from("Internal Server Error")),
            Self::BodyParsing(message) => (StatusCode::BAD_REQUEST, format!("Bad request error: {}", message)),
            Self::Indexer(err) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Indexer error: {}", err)),
        };
        (status, Json(json!({ "message": err_msg }))).into_response()
    }
}
