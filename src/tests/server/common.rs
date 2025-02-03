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
use crate::domain::models::indexer::{IndexerModel, IndexerStatus};
use crate::domain::models::types::AxumErrorResponse;
use crate::handlers::indexers::fail_indexer::fail_indexer;
use crate::routes::app_router;
use crate::tests::common::constants::{BROKEN_APIBARA_SCRIPT, WEHBHOOK_URL, WORKING_APIBARA_SCRIPT};
use crate::tests::common::utils::{
    get_indexer, get_indexers, is_process_running, send_create_indexer_request, send_create_webhook_indexer_request,
    send_delete_indexer_request, send_start_indexer_request, send_stop_indexer_request,
};
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
    let _config = config().await;

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
async fn start_two_indexers(#[future] setup_server: SocketAddr) {
    let addr = setup_server.await;

    let client = hyper::Client::new();
    let _config = config().await;

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
    assert!(is_process_running(indexer.process_id.unwrap()).await);

    // Create another indexer
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
    assert!(is_process_running(indexer.process_id.unwrap()).await);

    let indexers = get_indexers().await;
    assert_eq!(indexers.len(), 2);
}

#[rstest]
#[tokio::test]
async fn failed_running_indexer(#[future] setup_server: SocketAddr) {
    let addr = setup_server.await;

    let client = hyper::Client::new();
    let _config = config().await;

    // Create indexer
    let response = send_create_webhook_indexer_request(client.clone(), BROKEN_APIBARA_SCRIPT, addr).await;

    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let body: IndexerModel = serde_json::from_slice(&body).unwrap();

    // start the indexer
    send_start_indexer_request(client.clone(), body.id, addr).await;

    // sleep for 2 seconds to let the indexer fail
    tokio::time::sleep(Duration::from_secs(2)).await;

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
    let _config = config().await;

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

    // check indexer is present in DB in failed stopping state
    let indexer = get_indexer(body.id).await;
    assert_eq!(indexer.id, body.id);
    assert_eq!(indexer.status, IndexerStatus::Stopped);
}

#[rstest]
#[tokio::test]
async fn get_indexer_test(#[future] setup_server: SocketAddr) {
    let addr = setup_server.await;

    let client = hyper::Client::new();

    // Create indexer
    let response = send_create_webhook_indexer_request(client.clone(), WORKING_APIBARA_SCRIPT, addr).await;

    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let body: IndexerModel = serde_json::from_slice(&body).unwrap();

    // get the indexer
    let response = client
        .request(
            Request::builder().uri(format!("http://{}/v1/indexers/{}", addr, body.id)).body(Body::empty()).unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let body: IndexerModel = serde_json::from_slice(&body).unwrap();
    assert_eq!(body.id, body.id);
}

#[rstest]
#[tokio::test]
async fn get_all_indexers_test(#[future] setup_server: SocketAddr) {
    let addr = setup_server.await;

    let client = hyper::Client::new();

    // Create indexer
    let response = send_create_webhook_indexer_request(client.clone(), WORKING_APIBARA_SCRIPT, addr).await;

    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let body: IndexerModel = serde_json::from_slice(&body).unwrap();

    // get the indexers
    let response = client
        .request(Request::builder().uri(format!("http://{}/v1/indexers", addr)).body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let response_body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let response_body: Vec<IndexerModel> = serde_json::from_slice(&response_body).unwrap();
    assert_eq!(response_body.len(), 1);
    assert_eq!(response_body[0].id, body.id);
}

#[rstest]
#[tokio::test]
async fn delete_indexer_test_works_only_when_stopped(#[future] setup_server: SocketAddr) {
    let addr = setup_server.await;

    let client = hyper::Client::new();

    // Create indexer
    let response = send_create_webhook_indexer_request(client.clone(), WORKING_APIBARA_SCRIPT, addr).await;

    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let body: IndexerModel = serde_json::from_slice(&body).unwrap();

    // stop the indexer
    send_stop_indexer_request(client.clone(), body.id, addr).await;

    // delete the indexer
    let response = send_delete_indexer_request(client.clone(), body.id, addr).await;
    assert_eq!(response.status(), StatusCode::OK);

    // check indexer is not present in DB
    let indexers = get_indexers().await;
    assert_eq!(indexers.len(), 0);
}

#[rstest]
#[tokio::test]
async fn test_delete_indexer_fail_if_not_stopped(#[future] setup_server: SocketAddr) {
    let addr = setup_server.await;

    let client = hyper::Client::new();

    // Create indexer
    let response = send_create_webhook_indexer_request(client.clone(), WORKING_APIBARA_SCRIPT, addr).await;

    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let body: IndexerModel = serde_json::from_slice(&body).unwrap();

    // start the indexer
    send_start_indexer_request(client.clone(), body.id, addr).await;

    // delete the indexer
    let response = client
        .request(
            Request::builder()
                .uri(format!("http://{}/v1/indexers/delete/{}", addr, body.id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);

    // check indexer is present in DB
    let indexer = get_indexer(body.id).await;
    assert_eq!(indexer.id, body.id);
    assert_eq!(indexer.status, IndexerStatus::Running);
}
