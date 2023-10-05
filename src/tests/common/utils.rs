use std::net::SocketAddr;
use std::process::Stdio;

use axum::http;
use axum::http::{Request, Response, StatusCode};
use diesel::{Connection, PgConnection, RunQueryDsl};
use hyper::client::HttpConnector;
use hyper::{Body, Client};
use mpart_async::client::MultipartRequest;
use mpart_async::filestream::FileStream;
use tokio::process::Command;
use uuid::Uuid;

use crate::config::config;
use crate::domain::models::indexer::IndexerModel;
use crate::infra::repositories::indexer_repository::{IndexerRepository, Repository};
use crate::tests::common::constants::{TABLE_NAME, WEHBHOOK_URL};

/// Clears the database in the specified db_url. It first closes all connections
/// to that database as without it we get an error. The db_url must be the root db url
/// or the postgres db (default) url. You cannot be connected to the database you want to
/// clear.
pub fn clear_db(db_url: &str, db_name: &str) {
    let mut conn = PgConnection::establish(db_url).expect("Cannot connect to postgres database.");

    let disconnect_users = format!(
        "SELECT pg_terminate_backend(pid)
            FROM pg_stat_activity
            WHERE datname = '{}';",
        db_name
    );

    RunQueryDsl::execute(diesel::sql_query(disconnect_users.as_str()), &mut conn).unwrap();

    let query = diesel::sql_query(format!("DROP DATABASE IF EXISTS {}", db_name).as_str());
    RunQueryDsl::execute(query, &mut conn)
        .unwrap_or_else(|e| panic!("Couldn't drop database {}, error: {}", db_name, e));
}

/// Sends a request to create the indexer with the specified multipart body.
/// Arguments
/// - client: The hyper client to use to send the request
/// - mpart: The multipart body to send
/// - addr: The address of the server to send the request to
pub async fn send_create_indexer_request(
    client: Client<HttpConnector>,
    mpart: MultipartRequest<FileStream>,
    addr: SocketAddr,
) -> Response<Body> {
    let response = client
        .request(
            Request::builder()
                .method(http::Method::POST)
                .header(http::header::CONTENT_TYPE, format!("multipart/form-data; boundary={}", mpart.get_boundary()))
                .uri(format!("http://{}/v1/indexers", addr))
                .body(Body::wrap_stream(mpart))
                .unwrap(),
        )
        .await
        .unwrap();

    response
}

/// Sends a request to create a webhook indexer with the specified script path.
/// Arguments
/// - client: The hyper client to use to send the request
/// - script_path: The path to the script to use for the indexer
/// - addr: The address of the server to send the request to
pub async fn send_create_webhook_indexer_request(
    client: Client<HttpConnector>,
    script_path: &str,
    addr: SocketAddr,
) -> Response<Body> {
    let mut mpart = MultipartRequest::default();

    mpart.add_file("script.js", script_path);
    mpart.add_field("target_url", WEHBHOOK_URL);
    mpart.add_field("indexer_type", "Webhook");

    let response = send_create_indexer_request(client, mpart, addr).await;

    // assert the response of the request
    assert_eq!(response.status(), StatusCode::OK);
    response
}

/// Sends a request to create a postgres indexer with the specified script path.
/// Arguments
/// - client: The hyper client to use to send the request
/// - script_path: The path to the script to use for the indexer
/// - addr: The address of the server to send the request to
pub async fn send_create_postgres_indexer_request(
    client: Client<HttpConnector>,
    script_path: &str,
    addr: SocketAddr,
) -> Response<Body> {
    let mut mpart = MultipartRequest::default();

    mpart.add_file("script.js", script_path);
    mpart.add_field("table_name", TABLE_NAME);
    mpart.add_field("indexer_type", "Postgres");

    let response = send_create_indexer_request(client, mpart, addr).await;

    // assert the response of the request
    assert_eq!(response.status(), StatusCode::OK);
    response
}

/// Sends a request to start the indexer with the specified script path.
/// Arguments
/// - client: The hyper client to use to send the request
/// - id: The id of the indexer to start
/// - addr: The address of the server to send the request to
pub async fn send_start_indexer_request(client: Client<HttpConnector>, id: Uuid, addr: SocketAddr) {
    let response = client
        .request(
            Request::builder()
                .method(http::Method::POST)
                .uri(format!("http://{}/v1/indexers/start/{}", addr, id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    // assert the response of the request
    assert_eq!(response.status(), StatusCode::OK);
}

/// Sends a request to stop the indexer with the specified script path.
/// Arguments
/// - client: The hyper client to use to send the request
/// - id: The id of the indexer to stop
/// - addr: The address of the server to send the request to
pub async fn send_stop_indexer_request(client: Client<HttpConnector>, id: Uuid, addr: SocketAddr) {
    let response = client
        .request(
            Request::builder()
                .method(http::Method::POST)
                .uri(format!("http://{}/v1/indexers/stop/{}", addr, id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    // assert the response of the request
    assert_eq!(response.status(), StatusCode::OK);
}

/// Asserts that a queue contains a message which has a body equal to the
/// id of the specified indexer
pub async fn assert_queue_contains_message_with_indexer_id(queue_url: &str, indexer_id: Uuid) {
    let config = config().await;
    let message = config.sqs_client().receive_message().queue_url(queue_url).send().await.unwrap();
    assert_eq!(message.messages.clone().unwrap().len(), 1);
    let message = message.messages().unwrap().get(0).unwrap();
    assert_eq!(message.body().unwrap(), indexer_id.to_string());
}

/// Assert that a s3 buckets contains a specified key
pub async fn assert_s3_contains_key(bucket: &str, key: &str) {
    let config = config().await;
    assert!(config.s3_client().get_object().bucket(bucket).key(key).send().await.is_ok());
}

/// Get an indexer of the specified id from the database
pub async fn get_indexer(id: Uuid) -> IndexerModel {
    let config = config().await;
    let repository = IndexerRepository::new(config.pool());
    repository.get(id).await.unwrap()
}

/// Check if a process is running using the process id
pub async fn is_process_running(process_id: i64) -> bool {
    Command::new("ps")
        // Silence  stdout and stderr
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .args([
            "-p",
            process_id.to_string().as_str(),
        ])
        .spawn()
        .expect("Could not check the indexer status")
        .wait()
        .await
        .unwrap()
        .success()
}
