use axum::extract::multipart::MultipartError;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use object_store::Error;
use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumString};
use uuid::Uuid;

use crate::domain::models::types::AxumErrorResponse;
use crate::grpc::apibara_sink_v1::GetStatusResponse;
use crate::infra::errors::InfraError;

#[derive(Clone, Default, Debug, PartialEq, EnumString, Serialize, Deserialize, Display, Copy)]
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
    Postgres,
    Console,
}

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct IndexerModel {
    pub id: Uuid,
    pub status: IndexerStatus,
    pub indexer_type: IndexerType,
    pub process_id: Option<i64>,
    pub target_url: Option<String>,
    pub table_name: Option<String>,
    pub status_server_port: Option<i32>,
    pub custom_connection_string: Option<String>,
    pub starting_block: Option<i64>,
    pub indexer_id: Option<String>,
}

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct IndexerServerStatus {
    pub status: i32,
    pub starting_block: Option<u64>,
    pub current_block: Option<u64>,
    pub head_block: Option<u64>,
    #[serde(rename = "reason")]
    pub reason_: Option<String>,
}

impl From<GetStatusResponse> for IndexerServerStatus {
    fn from(value: GetStatusResponse) -> Self {
        Self {
            status: value.status,
            starting_block: value.starting_block,
            current_block: value.current_block,
            head_block: value.head_block,
            reason_: value.reason,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum IndexerError {
    #[error("internal server error: {0}")]
    InternalServerError(String),
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
    #[error("failed to stop indexer : {0}")]
    FailedToStopIndexer(i64),
    #[error("failed to start indexer : {0} (id: {1})")]
    FailedToStartIndexer(String, String),
    #[error("failed to upload to object_store")]
    FailedToUploadToStore(Error),
    #[error("failed to get from object_store")]
    FailedToGetFromStore(Error),
    #[error("failed to get collect bytes from object_store")]
    FailedToCollectBytesFromStore(Error),
    #[error("invalid indexer status")]
    InvalidIndexerStatus(IndexerStatus),
    #[error("failed to query db")]
    FailedToQueryDb(diesel::result::Error),
    #[error("invalid indexer type {0}")]
    InvalidIndexerType(String),
    #[error("failed to serialize {0}")]
    FailedToSerialize(String),
    #[error("indexer status server port not found")]
    IndexerStatusServerPortNotFound,
    #[error("failed to connect to gRPC server")]
    FailedToConnectGRPC(tonic::transport::Error),
    #[error("gRPC request failed")]
    GRPCRequestFailed(tonic::Status),
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
            _ => (StatusCode::INTERNAL_SERVER_ERROR, format!("Internal server error: {}", self)),
        };
        (
            status,
            Json(AxumErrorResponse {
                resource: "IndexerModel".into(),
                message: err_msg,
                happened_at: chrono::Utc::now(),
            }),
        )
            .into_response()
    }
}
