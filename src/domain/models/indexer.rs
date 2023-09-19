use axum::extract::multipart::MultipartError;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde::{Deserialize, Serialize};
use serde_json::json;
use strum_macros::{Display, EnumString};
use uuid::Uuid;

use crate::infra::errors::InfraError;

#[derive(Clone, Debug, PartialEq, EnumString, Serialize, Deserialize, Display)]
pub enum IndexerStatus {
    Created,
    Running,
    Stopped,
    Failed,
}

#[derive(Clone, Debug, PartialEq, EnumString, Serialize, Deserialize, Display)]
pub enum IndexerType {
    Webhook,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct IndexerModel {
    pub id: Uuid,
    pub status: IndexerStatus,
    pub indexer_type: IndexerType,
    pub process_id: Option<i32>,
}

#[derive(Debug)]
pub enum IndexerError {
    InternalServerError,
    NotFound(Uuid),
    InfraError(InfraError),
    FailedToReadFile(MultipartError),
    FailedToCreateFile(std::io::Error),
    IncorrectFileName,
    FailedToPushToQueue(aws_sdk_sqs::Error),
}

impl IntoResponse for IndexerError {
    fn into_response(self) -> axum::response::Response {
        tracing::error!("Error: {:?}", self);
        let (status, err_msg) = match self {
            Self::NotFound(id) => (StatusCode::NOT_FOUND, format!("IndexerModel with id {} has not been found", id)),
            Self::InfraError(db_error) => {
                (StatusCode::INTERNAL_SERVER_ERROR, format!("Internal server error: {}", db_error))
            }
            Self::IncorrectFileName => (StatusCode::BAD_REQUEST, format!("File key should be script.js")),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, format!("Internal server error")),
        };
        (status, Json(json!({"resource":"IndexerModel", "message": err_msg, "happened_at" : chrono::Utc::now() })))
            .into_response()
    }
}
