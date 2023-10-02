use std::net::{SocketAddr, TcpListener};
use std::sync::Arc;

use axum::body::Body;
use axum::http::{self, Request, StatusCode};
use mpart_async::client::MultipartRequest;
use rstest::*;
use tower::ServiceExt;

use crate::config::config;
use crate::domain::models::indexer::{IndexerModel, IndexerStatus, IndexerType};
use crate::routes::app_router;
use crate::AppState;

#[fixture]
async fn setup_server() -> SocketAddr {
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
async fn health() {
    let config = config().await;
    let state = AppState { pool: Arc::clone(config.pool()) };
    let app = app_router(state.clone()).with_state(state);

    // `Router` implements `tower::Service<Request<Body>>` so we can
    // call it like any tower service, no need to run an HTTP server.
    let response = app.oneshot(Request::builder().uri("/health").body(Body::empty()).unwrap()).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    assert!(body.is_empty());
}

#[rstest]
#[tokio::test]
async fn create_indexer() {
    let config = config().await;
    let state = AppState { pool: Arc::clone(config.pool()) };
    let app = app_router(state.clone()).with_state(state);

    // Create a multipart request
    let mut mpart = MultipartRequest::default();

    mpart.add_file("script.js", "./src/tests/scripts/test.js");
    mpart.add_field("webhook_url", "https://webhook.site/bc2ca42e-a8b2-43cf-b95c-779fb1a6bbbb");

    let response = app
        .oneshot(
            Request::builder()
                .method(http::Method::POST)
                .uri("/v1/indexers")
                .header(http::header::CONTENT_TYPE, format!("multipart/form-data; boundary={}", mpart.get_boundary()))
                .body(Body::wrap_stream(mpart))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let body: IndexerModel = serde_json::from_slice(&body).unwrap();

    // You can add more assertions based on the expected IndexerModel
    assert_eq!(body.indexer_type, IndexerType::Webhook);
    assert_eq!(body.target_url, "https://webhook.site/bc2ca42e-a8b2-43cf-b95c-779fb1a6bbbb");
}

#[rstest]
#[tokio::test]
async fn not_found() {
    let config = config().await;
    let state = AppState { pool: Arc::clone(config.pool()) };
    let app = app_router(state.clone()).with_state(state);

    let response = app.oneshot(Request::builder().uri("/does-not-exist").body(Body::empty()).unwrap()).await.unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    assert_eq!(&body[..], b"The requested resource was not found");
}

#[rstest]
#[tokio::test]
async fn test_health(#[future] setup_server: SocketAddr) {
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
async fn test_create_indexer(#[future] setup_server: SocketAddr) {
    let addr = setup_server.await;

    let client = hyper::Client::new();

    // Create a multipart request
    let mut mpart = MultipartRequest::default();

    mpart.add_file("script.js", "./src/tests/scripts/test.js");
    mpart.add_field("webhook_url", "https://webhook.site/bc2ca42e-a8b2-43cf-b95c-779fb1a6bbbb");

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

    assert_eq!(response.status(), StatusCode::OK);

    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let body: IndexerModel = serde_json::from_slice(&body).unwrap();

    assert_eq!(body.status, IndexerStatus::Created);
    assert_eq!(body.indexer_type, IndexerType::Webhook);
    assert_eq!(body.target_url, "https://webhook.site/bc2ca42e-a8b2-43cf-b95c-779fb1a6bbbb");
}
