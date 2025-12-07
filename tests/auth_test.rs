//! Integration tests for authentication middleware.
//!
//! These tests verify API key validation and origin checking.

use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode},
    middleware,
    routing::get,
};
use mcp_test_server::{Config, auth::auth_middleware};
use tower::ServiceExt; // for `oneshot`

async fn protected_handler() -> &'static str {
    "Protected resource"
}

async fn health_handler() -> &'static str {
    "OK"
}

fn create_app(config: Config) -> Router {
    let protected_routes = Router::new()
        .route("/protected", get(protected_handler))
        .layer(middleware::from_fn_with_state(config, auth_middleware));

    Router::new()
        .route("/health", get(health_handler))
        .merge(protected_routes)
}

#[tokio::test]
async fn test_health_endpoint_no_auth_required() {
    let config = Config::default();
    let app = create_app(config);

    let request = Request::builder()
        .uri("/health")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_protected_endpoint_without_api_key_config() {
    // No API key configured - should allow access
    let config = Config::default();
    let app = create_app(config);

    let request = Request::builder()
        .uri("/protected")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_protected_endpoint_missing_auth_header() {
    // API key configured but no header provided
    let config = Config {
        api_key: Some("test-secret-key".to_string()),
        ..Default::default()
    };
    let app = create_app(config);

    let request = Request::builder()
        .uri("/protected")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::FORBIDDEN);

    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body = String::from_utf8(body_bytes.to_vec()).unwrap();
    assert!(body.contains("forbidden"));
    assert!(body.contains("Missing Authorization header"));
}

#[tokio::test]
async fn test_protected_endpoint_invalid_auth_format() {
    let config = Config {
        api_key: Some("test-secret-key".to_string()),
        ..Default::default()
    };
    let app = create_app(config);

    let request = Request::builder()
        .uri("/protected")
        .header("Authorization", "Basic invalid")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::FORBIDDEN);

    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body = String::from_utf8(body_bytes.to_vec()).unwrap();
    assert!(body.contains("Invalid Authorization header format"));
}

#[tokio::test]
async fn test_protected_endpoint_wrong_api_key() {
    let config = Config {
        api_key: Some("test-secret-key".to_string()),
        ..Default::default()
    };
    let app = create_app(config);

    let request = Request::builder()
        .uri("/protected")
        .header("Authorization", "Bearer wrong-key")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::FORBIDDEN);

    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body = String::from_utf8(body_bytes.to_vec()).unwrap();
    assert!(body.contains("Invalid API key"));
}

#[tokio::test]
async fn test_protected_endpoint_correct_api_key() {
    let config = Config {
        api_key: Some("test-secret-key".to_string()),
        ..Default::default()
    };
    let app = create_app(config);

    let request = Request::builder()
        .uri("/protected")
        .header("Authorization", "Bearer test-secret-key")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_origin_validation_localhost_http() {
    let config = Config::default();
    let app = create_app(config);

    let request = Request::builder()
        .uri("/protected")
        .header("Origin", "http://localhost:3000")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_origin_validation_localhost_https() {
    let config = Config::default();
    let app = create_app(config);

    let request = Request::builder()
        .uri("/protected")
        .header("Origin", "https://localhost:3000")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_origin_validation_127_0_0_1() {
    let config = Config::default();
    let app = create_app(config);

    let request = Request::builder()
        .uri("/protected")
        .header("Origin", "http://127.0.0.1:8080")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_origin_validation_evil_domain() {
    let config = Config::default();
    let app = create_app(config);

    let request = Request::builder()
        .uri("/protected")
        .header("Origin", "http://evil.com")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::FORBIDDEN);

    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body = String::from_utf8(body_bytes.to_vec()).unwrap();
    assert!(body.contains("Origin not allowed"));
}

#[tokio::test]
async fn test_no_origin_header_allowed() {
    // Non-browser clients may not send Origin header
    let config = Config::default();
    let app = create_app(config);

    let request = Request::builder()
        .uri("/protected")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_combined_auth_and_origin() {
    let config = Config {
        api_key: Some("test-secret-key".to_string()),
        ..Default::default()
    };
    let app = create_app(config);

    let request = Request::builder()
        .uri("/protected")
        .header("Authorization", "Bearer test-secret-key")
        .header("Origin", "http://localhost:3000")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_combined_auth_correct_but_bad_origin() {
    let config = Config {
        api_key: Some("test-secret-key".to_string()),
        ..Default::default()
    };
    let app = create_app(config);

    let request = Request::builder()
        .uri("/protected")
        .header("Authorization", "Bearer test-secret-key")
        .header("Origin", "http://evil.com")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::FORBIDDEN);

    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body = String::from_utf8(body_bytes.to_vec()).unwrap();
    assert!(body.contains("Origin not allowed"));
}
