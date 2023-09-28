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
use sqs_worker::SQSListenerClientBuilderError;
use strum_macros::{Display, EnumString};
use uuid::Uuid;

use crate::infra::errors::InfraError;

#[derive(Clone, Default, Debug, PartialEq, EnumString, Serialize, Deserialize, Display)]
pub enum IndexerStatus {
    #[default]
    Created,
    Running,
    Stopped,
    FailedRunning,
    FailedStopping,
}

#[derive(Clone, Default, Debug, PartialEq, EnumString, Serialize, Deserialize, Display)]
pub enum IndexerType {
    #[default]
    Webhook,
}

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct IndexerModel {
    pub id: Uuid,
    pub status: IndexerStatus,
    pub indexer_type: IndexerType,
    pub process_id: Option<i64>,
    pub target_url: String,
}

#[derive(Debug, thiserror::Error)]
pub enum IndexerError {
    #[error("internal server error")]
    InternalServerError,
    #[error("infra error : {0}")]
    InfraError(InfraError),
    #[error("failed to read file from multipart request")]
    FailedToReadMultipartField(MultipartError),
    #[error("unexpected field in multipart request : {0}")]
    UnexpectedMultipartField(String),
    #[error("failed to build create indexer request")]
    FailedToBuildCreateIndexerRequest,
    #[error("failed to create file : {0}")]
    FailedToCreateFile(std::io::Error),
    #[error("failed to push to queue")]
    FailedToPushToQueue(aws_sdk_sqs::Error),
    #[error("failed to stop indexer : {0}")]
    FailedToStopIndexer(i64),
    #[error("failed to upload to S3")]
    FailedToUploadToS3(SdkError<PutObjectError>),
    #[error("failed to get from S3")]
    FailedToGetFromS3(SdkError<GetObjectError>),
    #[error("failed to collect bytes from S3")]
    FailedToCollectBytesFromS3(ByteStreamError),
    #[error("failed to create SQS listener")]
    FailedToCreateSQSListener(SQSListenerClientBuilderError),
    #[error("invalid indexer status")]
    InvalidIndexerStatus(IndexerStatus),
    #[error("failed to query db")]
    FailedToQueryDb(diesel::result::Error),
}

impl From<diesel::result::Error> for IndexerError {
    fn from(value: diesel::result::Error) -> Self {
        Self::FailedToQueryDb(value)
    }
}

impl IntoResponse for IndexerError {
    fn into_response(self) -> axum::response::Response {
        tracing::error!("Error: {:?}", self);
        let (status, err_msg) = match self {
            Self::InfraError(db_error) => {
                (StatusCode::INTERNAL_SERVER_ERROR, format!("Internal server error: {}", db_error))
            }
            _ => (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".into()),
        };
        (status, Json(json!({"resource":"IndexerModel", "message": err_msg, "happened_at" : chrono::Utc::now() })))
            .into_response()
    }
}
