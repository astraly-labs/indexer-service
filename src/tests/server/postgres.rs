use std::net::SocketAddr;

use hyper::StatusCode;
use mpart_async::client::MultipartRequest;
use rstest::rstest;

use crate::config::config;
use crate::constants::s3::INDEXER_SERVICE_BUCKET;
use crate::constants::sqs::START_INDEXER_QUEUE;
use crate::domain::models::indexer::{IndexerModel, IndexerStatus, IndexerType};
use crate::domain::models::types::AxumErrorResponse;
use crate::handlers::indexers::utils::get_s3_script_key;
use crate::tests::common::constants::{TABLE_NAME, WORKING_APIBARA_SCRIPT};
use crate::tests::common::utils::{
    assert_queue_contains_message_with_indexer_id, assert_s3_contains_key, get_indexer, send_create_indexer_request,
    send_create_postgres_indexer_request,
};
use crate::tests::server::common::setup_server;

#[rstest]
#[tokio::test]
async fn create_postgres_indexer(#[future] setup_server: SocketAddr) {
    let addr = setup_server.await;

    let client = hyper::Client::new();
    let config = config().await;

    // clear the sqs queue
    config.sqs_client().purge_queue().queue_url(START_INDEXER_QUEUE).send().await.unwrap();

    // Create indexer
    let response = send_create_postgres_indexer_request(client.clone(), WORKING_APIBARA_SCRIPT, addr).await;

    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let body: IndexerModel = serde_json::from_slice(&body).unwrap();

    assert_eq!(body.status, IndexerStatus::Created);
    assert_eq!(body.indexer_type, IndexerType::Postgres);
    assert_eq!(body.table_name, Some(TABLE_NAME.into()));
    assert_eq!(body.indexer_type, IndexerType::Postgres);

    // check if the file exists on S3
    assert_s3_contains_key(INDEXER_SERVICE_BUCKET, get_s3_script_key(body.id).as_str()).await;

    // check if the message is present on the queue
    assert_queue_contains_message_with_indexer_id(START_INDEXER_QUEUE, body.id).await;

    // check indexer is present in DB in created state
    let indexer = get_indexer(body.id).await;
    assert_eq!(indexer.id, body.id);
    assert_eq!(indexer.status, IndexerStatus::Created);
}

#[rstest]
#[tokio::test]
async fn create_postgres_indexer_fails_no_table_name(#[future] setup_server: SocketAddr) {
    let addr = setup_server.await;

    let client = hyper::Client::new();

    // Create indexer
    let mut mpart = MultipartRequest::default();

    mpart.add_file("script.js", WORKING_APIBARA_SCRIPT);
    mpart.add_field("indexer_type", "Postgres");
    let response = send_create_indexer_request(client.clone(), mpart, addr).await;

    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let body: AxumErrorResponse = serde_json::from_slice(&body).unwrap();
    assert_eq!(body.message, "Internal server error: failed to build create indexer request")
}
