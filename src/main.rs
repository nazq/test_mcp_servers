//! MCP Test Server entry point.

use mcp_test_server::{Config, McpTestServer};
use tracing_subscriber::{EnvFilter, fmt, prelude::*};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    let filter =
        EnvFilter::try_from_env("MCP_LOG_LEVEL").unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(filter)
        .init();

    // Load configuration
    let config = Config::from_env();

    tracing::info!(
        host = %config.host,
        port = config.port,
        "Starting MCP Test Server"
    );

    // Create and run server
    let server = McpTestServer::new(config);
    server.run().await?;

    Ok(())
}
