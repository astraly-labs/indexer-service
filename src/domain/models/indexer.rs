use aws_sdk_s3::error::SdkError;
use aws_sdk_s3::operation::get_object::GetObjectError;
use aws_sdk_s3::operation::put_object::PutObjectError;
use aws_sdk_s3::primitives::ByteStreamError;
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
    FailedRunning,
    FailedStopping,
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
    pub process_id: Option<i64>,
}

#[derive(Debug)]
pub enum IndexerError {
    InternalServerError,
    InfraError(InfraError),
    FailedToReadFile(MultipartError),
    FailedToCreateFile(std::io::Error),
    IncorrectFileName,
    FailedToPushToQueue(aws_sdk_sqs::Error),
    FailedToStopIndexer,
    FailedToUploadToS3(SdkError<PutObjectError>),
    FailedToGetFromS3(SdkError<GetObjectError>),
    FailedToCollectBytesFromS3(ByteStreamError),
}

impl IntoResponse for IndexerError {
    fn into_response(self) -> axum::response::Response {
        tracing::error!("Error: {:?}", self);
        let (status, err_msg) = match self {
            Self::InfraError(db_error) => {
                (StatusCode::INTERNAL_SERVER_ERROR, format!("Internal server error: {}", db_error))
            }
            Self::IncorrectFileName => (StatusCode::BAD_REQUEST, "File key should be script.js".into()),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".into()),
        };
        (status, Json(json!({"resource":"IndexerModel", "message": err_msg, "happened_at" : chrono::Utc::now() })))
            .into_response()
    }
}
