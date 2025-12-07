//! Lifecycle tests: server startup, health check, and shutdown.

mod common;

use common::TestServer;

#[tokio::test]
async fn test_server_starts_and_responds_to_health_check() {
    common::init_test_tracing();

    let server = TestServer::start().await;

    let client = reqwest::Client::new();
    let response = client.get(server.health_url()).send().await.unwrap();

    assert_eq!(response.status(), reqwest::StatusCode::OK);

    let body: serde_json::Value = response.json().await.unwrap();
    assert_eq!(body["status"], "ok");
}

#[tokio::test]
async fn test_server_handles_multiple_concurrent_health_checks() {
    common::init_test_tracing();

    let server = TestServer::start().await;
    let client = reqwest::Client::new();

    // Send 10 concurrent requests
    let mut handles = Vec::new();
    for _ in 0..10 {
        let client = client.clone();
        let url = server.health_url();
        handles.push(tokio::spawn(async move { client.get(url).send().await }));
    }

    // All should succeed
    for handle in handles {
        let response = handle.await.unwrap().unwrap();
        assert_eq!(response.status(), reqwest::StatusCode::OK);
    }
}

#[tokio::test]
async fn test_sse_endpoint_exists() {
    common::init_test_tracing();

    let server = TestServer::start().await;
    let client = reqwest::Client::new();

    // SSE endpoint should respond (even if connection isn't fully established in test)
    let response = client
        .get(server.sse_url())
        .timeout(std::time::Duration::from_secs(1))
        .send()
        .await;

    // Should either succeed or timeout waiting for SSE events - not return 404
    match response {
        Ok(r) => {
            // Should be 200 OK with SSE content type
            assert_eq!(r.status(), reqwest::StatusCode::OK);
            let content_type = r.headers().get("content-type").unwrap();
            assert!(content_type.to_str().unwrap().contains("text/event-stream"));
        }
        Err(e) => {
            // Timeout is acceptable for SSE endpoint (waiting for events)
            assert!(e.is_timeout() || e.is_connect(), "Unexpected error: {e}");
        }
    }
}

#[tokio::test]
async fn test_mcp_endpoint_exists() {
    common::init_test_tracing();

    let server = TestServer::start().await;
    let client = reqwest::Client::new();

    // POST to MCP endpoint without proper MCP message should return an error but not 404
    let response = client
        .post(server.mcp_url())
        .header("Content-Type", "application/json")
        .body("{}")
        .send()
        .await
        .unwrap();

    // Should not be 404 - the endpoint exists
    assert_ne!(response.status(), reqwest::StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_server_shutdown_is_clean() {
    common::init_test_tracing();

    let server = TestServer::start().await;

    // Verify server is running
    let client = reqwest::Client::new();
    let response = client.get(server.health_url()).send().await.unwrap();
    assert_eq!(response.status(), reqwest::StatusCode::OK);

    // Drop server (triggers shutdown)
    drop(server);

    // Give it a moment to shut down
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    // Test passes if we get here without panics
}
