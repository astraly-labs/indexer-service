use std::net::{SocketAddr, TcpListener};
use std::path::Path;
use std::sync::Arc;

use axum::body::Body;
use axum::extract::connect_info::MockConnectInfo;
use axum::http::{self, Request, StatusCode};
use tower::Service; // for `call`
use tower::ServiceExt;

use crate::config::config;
use crate::domain::models::indexer::{IndexerModel, IndexerType};
use crate::routes::app_router;
use crate::tests::common::multipart::Streamer;
use crate::AppState;

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
    assert_eq!(&body[..], b"");
}

#[tokio::test]
async fn create_indexer() {
    let config = config().await;
    let state = AppState { pool: Arc::clone(config.pool()) };
    let app = app_router(state.clone()).with_state(state);

    // Create a multipart request
    let file = std::fs::File::open("./src/tests/scripts/test.js").unwrap();
    let mut streaming = Streamer::new(file);
    // Apibara Script
    streaming.meta.set_name("script.js"); // field name 
    streaming.meta.set_filename("test.js"); // file name
    // Webhook url
    streaming.meta.set_name("webhook_url"); // field name 
    streaming.meta.set_filename("https://webhook.site/bc2ca42e-a8b2-43cf-b95c-779fb1a6bbbb"); // file name

    let body: Body = streaming.streaming();

    let response = app
        .oneshot(
            Request::builder()
                .method(http::Method::POST)
                .uri("/v1/indexers")
                .header(http::header::CONTENT_TYPE, mime::MULTIPART_FORM_DATA.as_ref())
                .body(body)
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

#[tokio::test]
async fn not_found() {
    let config = config().await;
    let state = AppState { pool: Arc::clone(config.pool()) };
    let app = app_router(state.clone()).with_state(state);

    let response = app.oneshot(Request::builder().uri("/does-not-exist").body(Body::empty()).unwrap()).await.unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    assert!(body.is_empty());
}

// You can also spawn a server and talk to it like any other HTTP server:
#[tokio::test]
async fn the_real_deal() {
    let config = config().await;
    let state = AppState { pool: Arc::clone(config.pool()) };
    let app = app_router(state.clone()).with_state(state);

    let listener = TcpListener::bind("0.0.0.0:0".parse::<SocketAddr>().unwrap()).unwrap();
    let addr = listener.local_addr().unwrap();

    tokio::spawn(async move {
        axum::Server::from_tcp(listener).unwrap().serve(app.into_make_service()).await.unwrap();
    });

    let client = hyper::Client::new();

    let response =
        client.request(Request::builder().uri(format!("http://{}", addr)).body(Body::empty()).unwrap()).await.unwrap();

    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    assert_eq!(&body[..], b"Hello, World!");
}

// You can use `ready()` and `call()` to avoid using `clone()`
// in multiple request
#[tokio::test]
async fn multiple_request() {
    let config = config().await;
    let state = AppState { pool: Arc::clone(config.pool()) };
    let mut app = app_router(state.clone()).with_state(state);

    let request = Request::builder().uri("/").body(Body::empty()).unwrap();
    let response = app.ready().await.unwrap().call(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let request = Request::builder().uri("/").body(Body::empty()).unwrap();
    let response = app.ready().await.unwrap().call(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

// Here we're calling `/requires-connect-into` which requires `ConnectInfo`
//
// That is normally set with `Router::into_make_service_with_connect_info` but we can't easily
// use that during tests. The solution is instead to set the `MockConnectInfo` layer during
// tests.
#[tokio::test]
async fn with_into_make_service_with_connect_info() {
    let config = config().await;
    let state = AppState { pool: Arc::clone(config.pool()) };
    let mut app =
        app_router(state.clone()).with_state(state).layer(MockConnectInfo(SocketAddr::from(([0, 0, 0, 0], 3000))));

    let request = Request::builder().uri("/requires-connect-into").body(Body::empty()).unwrap();
    let response = app.ready().await.unwrap().call(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}
