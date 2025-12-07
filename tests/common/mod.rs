//! Common test utilities for MCP test server integration tests.

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
pub struct TestServer {
    pub addr: SocketAddr,
    handle: JoinHandle<()>,
}

impl TestServer {
    /// Start a test server on an available port.
    pub async fn start() -> Self {
        Self::start_with_config(Config::default()).await
    }

    /// Start a test server with custom configuration.
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

    /// Get the SSE endpoint URL.
    #[must_use]
    pub fn sse_url(&self) -> String {
        format!("{}/sse", self.base_url())
    }

    /// Get the MCP streamable HTTP endpoint URL.
    #[must_use]
    pub fn mcp_url(&self) -> String {
        format!("{}/mcp", self.base_url())
    }
}

impl Drop for TestServer {
    fn drop(&mut self) {
        self.handle.abort();
    }
}

/// Initialize tracing for tests.
pub fn init_test_tracing() {
    let _ = tracing_subscriber::fmt()
        .with_test_writer()
        .with_max_level(tracing::Level::DEBUG)
        .try_init();
}
