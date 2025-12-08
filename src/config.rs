//! Configuration management for the MCP Test Server.

use std::env;
use std::net::IpAddr;

/// Server configuration loaded from environment variables.
#[derive(Debug, Clone)]
pub struct Config {
    /// Server bind address (default: 0.0.0.0)
    pub host: IpAddr,
    /// Server listen port (default: 3000)
    pub port: u16,
    /// Optional API key for authentication
    pub api_key: Option<String>,
    /// Log level (default: info)
    pub log_level: String,
}

impl Config {
    /// Create a new configuration builder.
    ///
    /// # Example
    ///
    /// ```
    /// use mcp_test_server::Config;
    ///
    /// let config = Config::builder()
    ///     .port(8080)
    ///     .api_key("secret")
    ///     .build();
    ///
    /// assert_eq!(config.port, 8080);
    /// assert!(config.requires_auth());
    /// ```
    #[must_use]
    pub fn builder() -> ConfigBuilder {
        ConfigBuilder::default()
    }

    /// Load configuration from environment variables.
    ///
    /// # Panics
    ///
    /// This function will panic if the default host address "0.0.0.0" fails to parse,
    /// which should never happen under normal circumstances.
    #[must_use]
    pub fn from_env() -> Self {
        Self {
            host: env::var("MCP_HOST")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or_else(|| "0.0.0.0".parse().unwrap()),
            port: env::var("MCP_PORT")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(3000),
            api_key: env::var("MCP_API_KEY").ok().filter(|s| !s.is_empty()),
            log_level: env::var("MCP_LOG_LEVEL").unwrap_or_else(|_| "info".to_string()),
        }
    }

    /// Check if authentication is required.
    #[must_use]
    pub const fn requires_auth(&self) -> bool {
        self.api_key.is_some()
    }
}

/// Builder for creating [`Config`] instances with a fluent API.
///
/// # Example
///
/// ```
/// use mcp_test_server::Config;
/// use std::net::IpAddr;
///
/// let config = Config::builder()
///     .host("127.0.0.1".parse::<IpAddr>().unwrap())
///     .port(9000)
///     .api_key("my-secret-key")
///     .log_level("debug")
///     .build();
/// ```
#[derive(Debug, Default)]
pub struct ConfigBuilder {
    host: Option<IpAddr>,
    port: Option<u16>,
    api_key: Option<String>,
    log_level: Option<String>,
}

impl ConfigBuilder {
    /// Set the server bind address.
    #[must_use]
    pub const fn host(mut self, host: IpAddr) -> Self {
        self.host = Some(host);
        self
    }

    /// Set the server listen port.
    #[must_use]
    pub const fn port(mut self, port: u16) -> Self {
        self.port = Some(port);
        self
    }

    /// Set the API key for authentication.
    #[must_use]
    pub fn api_key(mut self, key: impl Into<String>) -> Self {
        self.api_key = Some(key.into());
        self
    }

    /// Set the log level.
    #[must_use]
    pub fn log_level(mut self, level: impl Into<String>) -> Self {
        self.log_level = Some(level.into());
        self
    }

    /// Build the configuration with defaults for unset values.
    ///
    /// # Panics
    ///
    /// This function will panic if the default host address "0.0.0.0" fails to parse,
    /// which should never happen under normal circumstances.
    #[must_use]
    pub fn build(self) -> Config {
        Config {
            host: self.host.unwrap_or_else(|| "0.0.0.0".parse().unwrap()),
            port: self.port.unwrap_or(3000),
            api_key: self.api_key,
            log_level: self.log_level.unwrap_or_else(|| "info".to_string()),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            host: "0.0.0.0".parse().unwrap(),
            port: 3000,
            api_key: None,
            log_level: "info".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.port, 3000);
        assert!(!config.requires_auth());
        assert_eq!(config.log_level, "info");
        assert_eq!(config.host.to_string(), "0.0.0.0");
    }

    #[test]
    fn test_config_with_api_key() {
        let config = Config {
            api_key: Some("test-key".to_string()),
            ..Default::default()
        };
        assert!(config.requires_auth());
    }

    #[test]
    fn test_config_no_api_key() {
        let config = Config::default();
        assert!(!config.requires_auth());
    }

    #[test]
    fn test_config_custom_host() {
        let config = Config {
            host: "127.0.0.1".parse().unwrap(),
            port: 8080,
            api_key: Some("secret".to_string()),
            log_level: "debug".to_string(),
        };
        assert_eq!(config.host.to_string(), "127.0.0.1");
        assert_eq!(config.port, 8080);
        assert!(config.requires_auth());
        assert_eq!(config.log_level, "debug");
    }

    #[test]
    fn test_config_clone() {
        use std::net::{IpAddr, Ipv4Addr};

        let config = Config {
            api_key: Some("test-key".to_string()),
            ..Default::default()
        };
        let cloned = config.clone();
        // Verify clone is independent
        drop(config);
        assert_eq!(cloned.port, 3000);
        assert_eq!(cloned.host, IpAddr::V4(Ipv4Addr::UNSPECIFIED));
        assert_eq!(cloned.api_key, Some("test-key".to_string()));
    }

    #[test]
    fn test_config_debug() {
        let config = Config::default();
        let debug_str = format!("{config:?}");
        assert!(debug_str.contains("Config"));
        assert!(debug_str.contains("3000"));
    }

    // Note: from_env() tests are skipped since env::set_var is unsafe in edition 2024
    // and requires unsafe blocks which are forbidden in this crate.
    // The from_env() function is tested indirectly through integration tests.

    // =============================================================================
    // BUILDER TESTS
    // =============================================================================

    #[test]
    fn test_builder_defaults() {
        let config = Config::builder().build();
        assert_eq!(config.port, 3000);
        assert_eq!(config.host.to_string(), "0.0.0.0");
        assert!(!config.requires_auth());
        assert_eq!(config.log_level, "info");
    }

    #[test]
    fn test_builder_with_port() {
        let config = Config::builder().port(8080).build();
        assert_eq!(config.port, 8080);
    }

    #[test]
    fn test_builder_with_host() {
        let config = Config::builder().host("127.0.0.1".parse().unwrap()).build();
        assert_eq!(config.host.to_string(), "127.0.0.1");
    }

    #[test]
    fn test_builder_with_api_key() {
        let config = Config::builder().api_key("test-key").build();
        assert!(config.requires_auth());
        assert_eq!(config.api_key, Some("test-key".to_string()));
    }

    #[test]
    fn test_builder_with_log_level() {
        let config = Config::builder().log_level("debug").build();
        assert_eq!(config.log_level, "debug");
    }

    #[test]
    fn test_builder_chaining() {
        let config = Config::builder()
            .host("127.0.0.1".parse().unwrap())
            .port(9000)
            .api_key("secret")
            .log_level("trace")
            .build();

        assert_eq!(config.host.to_string(), "127.0.0.1");
        assert_eq!(config.port, 9000);
        assert_eq!(config.api_key, Some("secret".to_string()));
        assert_eq!(config.log_level, "trace");
    }

    #[test]
    fn test_builder_debug() {
        let builder = Config::builder().port(8080);
        let debug_str = format!("{builder:?}");
        assert!(debug_str.contains("ConfigBuilder"));
        assert!(debug_str.contains("8080"));
    }
}
