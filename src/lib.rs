//! MCP Test Server - A full-featured Model Context Protocol test server.
//!
//! This crate provides a complete MCP server implementation for integration testing
//! of MCP clients. It uses Streamable HTTP transport.
//!
//! Available as a Docker image: `ghcr.io/nazq/mcp-test-server:latest`
//!
//! # Purpose
//!
//! **Designed for MCP client developers** who need to thoroughly test their implementations.
//! Use with [Testcontainers](https://testcontainers.com/) or similar tools to spin up
//! ephemeral test servers in your CI/CD pipelines or local development environment.
//!
//! # Features
//!
//! - Full MCP 2025-11-25 specification compliance
//! - Streamable HTTP transport (`/mcp` endpoint)
//! - API key authentication via `Authorization: Bearer` header
//! - 26 tools for testing (math, string, encoding, utility, testing)
//! - 8 resources (static and dynamic) with subscription support
//! - 5 prompts with argument validation
//! - Auto-completion for prompt arguments
//! - Logging level control
//!
//! # Quick Start
//!
//! ```rust,no_run
//! use mcp_test_server::{Config, McpTestServer};
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let config = Config::default();
//!     let server = McpTestServer::new(config);
//!     server.run().await
//! }
//! ```
//!
//! # Configuration
//!
//! Configuration is done via environment variables:
//!
//! | Variable | Default | Description |
//! |----------|---------|-------------|
//! | `MCP_HOST` | `0.0.0.0` | Server bind address |
//! | `MCP_PORT` | `3000` | Server listen port |
//! | `MCP_API_KEY` | (none) | API key for authentication |
//! | `MCP_LOG_LEVEL` | `info` | Logging level |
//!
//! # Modules
//!
//! - [`auth`] - Authentication middleware for API key and origin validation
//! - [`config`] - Server configuration from environment variables
//! - [`prompts`] - Prompt templates and argument handling
//! - [`resources`] - Static and dynamic resource handlers
//! - [`server`] - Main server implementation with all tools
//! - [`tools`] - Tool parameter structures

pub mod auth;
pub mod config;
pub mod error;
pub mod icons;
pub mod prompts;
pub mod resources;
pub mod server;
pub mod tools;

pub use config::Config;
pub use error::{Result, ServerError};
pub use resources::ResourceHandler;
pub use server::McpTestServer;
