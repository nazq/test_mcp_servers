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
        let config = Config::default();
        let cloned = config.clone();
        assert_eq!(config.port, cloned.port);
        assert_eq!(config.host, cloned.host);
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
}
