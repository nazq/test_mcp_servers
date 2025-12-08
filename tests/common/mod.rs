//! Common test utilities for MCP test server integration tests.
//!
//! This module provides test helpers for spinning up ephemeral MCP test servers
//! and making requests against them.

#![allow(dead_code)] // Test helpers may not all be used in every test file

use std::net::SocketAddr;
use std::sync::atomic::{AtomicU16, Ordering};
use std::time::Duration;

use mcp_test_server::{Config, McpTestServer};
use tokio::task::JoinHandle;

/// Starting port for tests to avoid conflicts.
static PORT_COUNTER: AtomicU16 = AtomicU16::new(39000);

/// Get a unique port for testing.
#[must_use]
pub fn get_test_port() -> u16 {
    PORT_COUNTER.fetch_add(1, Ordering::SeqCst)
}

/// Test server handle that automatically shuts down when dropped.
///
/// # Example
///
/// ```ignore
/// let server = TestServer::start().await;
/// let client = reqwest::Client::new();
/// let resp = client.get(server.health_url()).send().await?;
/// assert_eq!(resp.status(), 200);
/// // Server automatically shuts down when `server` is dropped
/// ```
pub struct TestServer {
    /// The socket address the server is bound to.
    pub addr: SocketAddr,
    handle: JoinHandle<()>,
}

impl TestServer {
    /// Start a test server on an available port with default configuration.
    pub async fn start() -> Self {
        Self::start_with_config(Config::default()).await
    }

    /// Start a test server with API key authentication enabled.
    pub async fn start_with_auth(api_key: impl Into<String>) -> Self {
        let config = Config::builder().api_key(api_key).build();
        Self::start_with_config(config).await
    }

    /// Start a test server with custom configuration.
    ///
    /// Note: The host and port will be overridden to use localhost
    /// and a unique test port.
    pub async fn start_with_config(mut config: Config) -> Self {
        let port = get_test_port();
        config.host = std::net::IpAddr::V4(std::net::Ipv4Addr::LOCALHOST);
        config.port = port;

        let server = McpTestServer::new(config);
        let addr = SocketAddr::new(server.config().host, server.config().port);

        let handle = tokio::spawn(async move {
            if let Err(e) = server.run().await {
                tracing::error!("Test server error: {e}");
            }
        });

        // Give server time to start
        tokio::time::sleep(Duration::from_millis(100)).await;

        Self { addr, handle }
    }

    /// Get the base URL for this test server.
    #[must_use]
    pub fn base_url(&self) -> String {
        format!("http://{}", self.addr)
    }

    /// Get the health endpoint URL.
    #[must_use]
    pub fn health_url(&self) -> String {
        format!("{}/health", self.base_url())
    }

    /// Get the MCP streamable HTTP endpoint URL.
    #[must_use]
    pub fn mcp_url(&self) -> String {
        format!("{}/mcp", self.base_url())
    }

    /// Get the server's port.
    #[must_use]
    pub const fn port(&self) -> u16 {
        self.addr.port()
    }
}

impl Drop for TestServer {
    fn drop(&mut self) {
        self.handle.abort();
    }
}

/// Initialize tracing for tests.
///
/// This function is safe to call multiple times; subsequent calls are no-ops.
pub fn init_test_tracing() {
    let _ = tracing_subscriber::fmt()
        .with_test_writer()
        .with_max_level(tracing::Level::DEBUG)
        .try_init();
}

/// Create a pre-configured reqwest client for testing.
#[must_use]
pub fn test_client() -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .expect("Failed to build test client")
}

/// Create a reqwest client with an authorization header.
#[must_use]
pub fn test_client_with_auth(api_key: &str) -> reqwest::Client {
    use reqwest::header::{AUTHORIZATION, HeaderMap, HeaderValue};

    let mut headers = HeaderMap::new();
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&format!("Bearer {api_key}")).expect("Invalid API key"),
    );

    reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .default_headers(headers)
        .build()
        .expect("Failed to build test client")
}
