use std::net::SocketAddr;
use std::process::Stdio;

use axum::http;
use axum::http::{Request, Response, StatusCode};
use diesel::{Connection, PgConnection, RunQueryDsl};
use hyper::client::HttpConnector;
use hyper::{Body, Client};
use mpart_async::client::MultipartRequest;
use tokio::process::Command;
use uuid::Uuid;

use crate::config::config;
use crate::constants::s3::INDEXER_SERVICE_BUCKET;
use crate::domain::models::indexer::IndexerModel;
use crate::handlers::indexers::utils::get_s3_script_key;
use crate::infra::repositories::indexer_repository::{IndexerRepository, Repository};
use crate::tests::common::constants::WEHBHOOK_URL;

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

pub async fn send_create_indexer_request(
    client: Client<HttpConnector>,
    script_path: &str,
    addr: SocketAddr,
) -> Response<Body> {
    let mut mpart = MultipartRequest::default();

    mpart.add_file("script.js", script_path);
    mpart.add_field("webhook_url", WEHBHOOK_URL);

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

    // assert the response of the request
    assert_eq!(response.status(), StatusCode::OK);
    response
}

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

pub async fn assert_queue_contains_message_with_indexer_id(queue_url: &str, indexer_id: Uuid) {
    let config = config().await;
    let message = config.sqs_client().receive_message().queue_url(queue_url).send().await.unwrap();
    assert_eq!(message.messages.clone().unwrap().len(), 1);
    let message = message.messages().unwrap().get(0).unwrap();
    assert_eq!(message.body().unwrap(), indexer_id.to_string());
}

pub async fn assert_s3_contains_key(bucket: &str, key: &str) {
    let config = config().await;
    assert!(
        config
            .s3_client()
            .get_object()
            .bucket(bucket)
            .key(key)
            .send()
            .await
            .is_ok()
    );
}

pub async fn get_indexer(id: Uuid) -> IndexerModel {
    let config = config().await;
    let repository = IndexerRepository::new(config.pool());
    repository.get(id).await.unwrap()
}

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
