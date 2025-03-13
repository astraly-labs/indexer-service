use std::net::SocketAddr;

use hyper::StatusCode;
use mpart_async::client::MultipartRequest;
use rstest::rstest;

use crate::config::config;
use crate::domain::models::indexer::{IndexerModel, IndexerStatus, IndexerType};
use crate::domain::models::types::AxumErrorResponse;
use crate::handlers::indexers::utils::get_s3_script_key;
use crate::tests::common::constants::{TABLE_NAME, TABLE_NAME_2, WORKING_APIBARA_SCRIPT};
use crate::tests::common::utils::{
    assert_store_contains_key, get_indexer, get_indexer_by_table_name, send_create_indexer_request,
    send_create_postgres_indexer_request,
};
use crate::tests::server::common::setup_server;

#[rstest]
#[tokio::test]
async fn create_postgres_indexer(#[future] setup_server: SocketAddr) {
    let addr = setup_server.await;

    let client = hyper::Client::new();
    let _config = config().await;

    // Create indexer
    let response = send_create_postgres_indexer_request(client.clone(), WORKING_APIBARA_SCRIPT, addr).await;

    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let body: IndexerModel = serde_json::from_slice(&body).unwrap();

    assert_eq!(body.status, IndexerStatus::Created);
    assert_eq!(body.indexer_type, IndexerType::Postgres);
    assert_eq!(body.table_name, Some(TABLE_NAME.into()));
    assert_eq!(body.indexer_type, IndexerType::Postgres);

    // check if the file exists in our object store
    assert_store_contains_key(get_s3_script_key(body.id).as_str()).await;

    // check indexer is present in DB in created state
    let indexer = get_indexer(body.id).await;
    assert_eq!(indexer.id, body.id);
    assert_eq!(indexer.status, IndexerStatus::Created);
    assert_eq!(indexer.indexer_type, IndexerType::Postgres);
    assert_eq!(indexer.table_name, Some(TABLE_NAME.into()));

    // check that we can get the indexer by table name
    let indexer = get_indexer_by_table_name(TABLE_NAME).await;
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

#[rstest]
#[tokio::test]
async fn create_multiple_postgres_indexers(#[future] setup_server: SocketAddr) {
    let addr = setup_server.await;
    let client = hyper::Client::new();

    // Create first indexer
    let response1 = send_create_postgres_indexer_request(client.clone(), WORKING_APIBARA_SCRIPT, addr).await;
    let body1 = hyper::body::to_bytes(response1.into_body()).await.unwrap();
    let indexer1: IndexerModel = serde_json::from_slice(&body1).unwrap();

    assert_eq!(indexer1.status, IndexerStatus::Created);
    assert_eq!(indexer1.indexer_type, IndexerType::Postgres);
    assert_eq!(indexer1.table_name, Some(TABLE_NAME.into()));

    // Create second indexer with different table name
    let mut mpart = MultipartRequest::default();
    mpart.add_file("script.js", WORKING_APIBARA_SCRIPT);
    mpart.add_field("indexer_type", "Postgres");
    mpart.add_field("table_name", TABLE_NAME_2);
    let response2 = send_create_indexer_request(client.clone(), mpart, addr).await;
    let body2 = hyper::body::to_bytes(response2.into_body()).await.unwrap();
    let indexer2: IndexerModel = serde_json::from_slice(&body2).unwrap();

    assert_eq!(indexer2.status, IndexerStatus::Created);
    assert_eq!(indexer2.indexer_type, IndexerType::Postgres);
    assert_eq!(indexer2.table_name, Some(TABLE_NAME_2.into()));

    // Verify both indexers exist and have different IDs
    assert_ne!(indexer1.id, indexer2.id);

    // Verify both files exist in object store
    assert_store_contains_key(get_s3_script_key(indexer1.id).as_str()).await;
    assert_store_contains_key(get_s3_script_key(indexer2.id).as_str()).await;

    // Verify both indexers are in DB in created state
    let db_indexer1 = get_indexer(indexer1.id).await;
    assert_eq!(db_indexer1.status, IndexerStatus::Created);
    assert_eq!(db_indexer1.table_name, Some(TABLE_NAME.into()));

    let db_indexer2 = get_indexer(indexer2.id).await;
    assert_eq!(db_indexer2.status, IndexerStatus::Created);
    assert_eq!(db_indexer2.table_name, Some(TABLE_NAME_2.into()));

    // Verify we can get both indexers by their table names
    let table1_indexer = get_indexer_by_table_name(TABLE_NAME).await;
    assert_eq!(table1_indexer.id, indexer1.id);

    let table2_indexer = get_indexer_by_table_name(TABLE_NAME_2).await;
    assert_eq!(table2_indexer.id, indexer2.id);
}
