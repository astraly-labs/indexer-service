use uuid::Uuid;

use crate::constants::s3::INDEXER_SERVICE_SCRIPTS_FOLDER;
use crate::domain::models::indexer::{IndexerError, IndexerServerStatus};
use crate::grpc::apibara_sink_v1::status_client::StatusClient;
use crate::grpc::apibara_sink_v1::GetStatusRequest;

pub fn get_s3_script_key(id: Uuid) -> String {
    format!("{}/{}.js", INDEXER_SERVICE_SCRIPTS_FOLDER, id)
}

pub fn get_script_tmp_directory(id: Uuid) -> String {
    format!("{}/{}.js", std::env::temp_dir().to_str().unwrap(), id)
}

pub async fn query_status_server(server_port: i32) -> Result<IndexerServerStatus, IndexerError> {
    // Create a gRPC client
    let endpoint = format!("http://localhost:{}", server_port);

    let mut client = StatusClient::connect(endpoint).await.map_err(IndexerError::FailedToConnectGRPC)?;

    // Create a GetStatusRequest
    let request = tonic::Request::new(GetStatusRequest {});

    // Fetch the status
    let response = client.get_status(request).await.map_err(IndexerError::GRPCRequestFailed)?;

    // Process the response
    let status_response = response.into_inner();

    Ok(status_response.into())
}
