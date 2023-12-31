use std::net::{SocketAddr, TcpListener};
use std::process::Stdio;
use std::sync::Arc;
use std::time::Duration;

use axum::http::StatusCode;
use hyper::{Body, Request};
use mpart_async::client::MultipartRequest;
use rstest::{fixture, rstest};
use tokio::process::Command;

use crate::config::{config, config_force_init};
use crate::constants::sqs::{START_INDEXER_QUEUE, STOP_INDEXER_QUEUE};
use crate::domain::models::indexer::{IndexerModel, IndexerStatus};
use crate::domain::models::types::AxumErrorResponse;
use crate::handlers::indexers::fail_indexer::fail_indexer;
use crate::routes::app_router;
use crate::tests::common::constants::{BROKEN_APIBARA_SCRIPT, WEHBHOOK_URL, WORKING_APIBARA_SCRIPT};
use crate::tests::common::utils::{
    assert_queue_contains_message_with_indexer_id, get_indexer, is_process_running, send_create_indexer_request,
    send_create_webhook_indexer_request, send_start_indexer_request, send_stop_indexer_request,
};
use crate::types::sqs::{StartIndexerRequest, StopIndexerRequest};
use crate::AppState;

#[fixture]
pub async fn setup_server() -> SocketAddr {
    config_force_init().await;
    let config = config().await;
    let state = AppState { pool: Arc::clone(config.pool()) };
    let app = app_router(state.clone()).with_state(state);

    let listener = TcpListener::bind("0.0.0.0:0".parse::<SocketAddr>().unwrap()).unwrap();
    let addr = listener.local_addr().unwrap();

    tokio::spawn(async move {
        axum::Server::from_tcp(listener).unwrap().serve(app.into_make_service()).await.unwrap();
    });

    addr
}

#[rstest]
#[tokio::test]
async fn not_found(#[future] setup_server: SocketAddr) {
    let addr = setup_server.await;

    let client = hyper::Client::new();

    let response = client
        .request(Request::builder().uri(format!("http://{}/does-not-exist", addr)).body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    assert_eq!(&body[..], b"The requested resource was not found");
}

#[rstest]
#[tokio::test]
async fn health(#[future] setup_server: SocketAddr) {
    let addr = setup_server.await;

    let client = hyper::Client::new();

    let response = client
        .request(Request::builder().uri(format!("http://{}/health", addr)).body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    assert!(body.is_empty());
}

#[rstest]
#[tokio::test]
async fn create_indexer_fails_no_script(#[future] setup_server: SocketAddr) {
    let addr = setup_server.await;

    let client = hyper::Client::new();

    // Create indexer
    let mut mpart = MultipartRequest::default();

    mpart.add_field("indexer_type", "Webhook");
    mpart.add_field("target_url", WEHBHOOK_URL);
    let response = send_create_indexer_request(client.clone(), mpart, addr).await;

    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let body: AxumErrorResponse = serde_json::from_slice(&body).unwrap();
    assert_eq!(body.message, "Internal server error: failed to build create indexer request")
}

#[rstest]
#[tokio::test]
async fn start_indexer(#[future] setup_server: SocketAddr) {
    let addr = setup_server.await;

    let client = hyper::Client::new();
    let config = config().await;

    // clear the sqs queue
    config.sqs_client().purge_queue().queue_url(START_INDEXER_QUEUE).send().await.unwrap();

    // Create indexer
    let response = send_create_webhook_indexer_request(client.clone(), WORKING_APIBARA_SCRIPT, addr).await;

    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let body: IndexerModel = serde_json::from_slice(&body).unwrap();

    // start the indexer
    send_start_indexer_request(client.clone(), body.id, addr).await;

    // check indexer is present in DB in running state
    let indexer = get_indexer(body.id).await;
    assert_eq!(indexer.id, body.id);
    assert_eq!(indexer.status, IndexerStatus::Running);

    // check the process is actually up
    assert!(is_process_running(indexer.process_id.unwrap()).await,);
}

#[rstest]
#[tokio::test]
async fn failed_running_indexer(#[future] setup_server: SocketAddr) {
    let addr = setup_server.await;

    let client = hyper::Client::new();
    let config = config().await;

    // clear the sqs queue
    config.sqs_client().purge_queue().queue_url(START_INDEXER_QUEUE).send().await.unwrap();

    // Create indexer
    let response = send_create_webhook_indexer_request(client.clone(), BROKEN_APIBARA_SCRIPT, addr).await;

    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let body: IndexerModel = serde_json::from_slice(&body).unwrap();

    // check if the message is present on the queue
    let mut request = StartIndexerRequest { id: body.id, attempt_no: 1 };
    assert_queue_contains_message_with_indexer_id(START_INDEXER_QUEUE, serde_json::to_string(&request).unwrap()).await;

    // start the indexer
    send_start_indexer_request(client.clone(), body.id, addr).await;

    // sleep for 2 seconds to let the indexer fail
    tokio::time::sleep(Duration::from_secs(2)).await;

    // check if the message is present on the queue
    request.attempt_no = 2;
    assert_queue_contains_message_with_indexer_id(START_INDEXER_QUEUE, serde_json::to_string(&request).unwrap()).await;

    // fail the indexer
    assert!(fail_indexer(body.id).await.is_ok());

    // check indexer is present in DB in failed running state state
    let indexer = get_indexer(body.id).await;
    assert_eq!(indexer.id, body.id);
    assert_eq!(indexer.status, IndexerStatus::FailedRunning);

    // check the process has exited
    assert!(!is_process_running(indexer.process_id.unwrap()).await);
}

// Ignoring this test case as it's flaky. Works locally fails on github actions.
#[rstest]
#[tokio::test]
#[ignore]
async fn stop_indexer(#[future] setup_server: SocketAddr) {
    let addr = setup_server.await;

    let client = hyper::Client::new();

    // Create indexer
    let response = send_create_webhook_indexer_request(client.clone(), WORKING_APIBARA_SCRIPT, addr).await;

    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let body: IndexerModel = serde_json::from_slice(&body).unwrap();

    // start the indexer
    send_start_indexer_request(client.clone(), body.id, addr).await;

    tokio::time::sleep(Duration::from_secs(2)).await;

    // stop the indexer
    send_stop_indexer_request(client.clone(), body.id, addr).await;

    // check indexer is present in DB in created state
    let indexer = get_indexer(body.id).await;
    assert_eq!(indexer.id, body.id);
    assert_eq!(indexer.status, IndexerStatus::Stopped);
}

// Ignoring this test case as it's flaky. Works locally fails on github actions.
#[rstest]
#[tokio::test]
async fn failed_stop_indexer(#[future] setup_server: SocketAddr) {
    let addr = setup_server.await;

    let client = hyper::Client::new();
    let config = config().await;

    // clear the sqs queue
    config.sqs_client().purge_queue().queue_url(STOP_INDEXER_QUEUE).send().await.unwrap();

    // Create indexer
    let response = send_create_webhook_indexer_request(client.clone(), WORKING_APIBARA_SCRIPT, addr).await;

    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let body: IndexerModel = serde_json::from_slice(&body).unwrap();

    // start the indexer
    send_start_indexer_request(client.clone(), body.id, addr).await;

    // sleep for 5 seconds to let the indexer start
    tokio::time::sleep(Duration::from_secs(5)).await;

    // kill indexer so stop fails
    let indexer = get_indexer(body.id).await;
    assert!(
        Command::new("kill")
            // Silence  stdout and stderr
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .args([
                indexer.process_id.unwrap().to_string().as_str(),
            ])
            .spawn()
            .expect("Could not stop the webhook indexer")
            .wait()
            .await
            .unwrap()
            .success()
    );

    // sleep for 2 seconds to let the message go to queue
    tokio::time::sleep(Duration::from_secs(2)).await;

    // stop the indexer
    send_stop_indexer_request(client.clone(), body.id, addr).await;

    // check if the message is present on the queue
    let request = StopIndexerRequest { id: body.id, status: IndexerStatus::Stopped };
    assert_queue_contains_message_with_indexer_id(STOP_INDEXER_QUEUE, serde_json::to_string(&request).unwrap()).await;

    // check indexer is present in DB in failed stopping state
    let indexer = get_indexer(body.id).await;
    assert_eq!(indexer.id, body.id);
    assert_eq!(indexer.status, IndexerStatus::Stopped);
}
