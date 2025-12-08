//! Unified error types for the MCP test server.
//!
//! This module provides a standardized error type hierarchy using `thiserror`
//! for type-safe error handling across the server.

use thiserror::Error;

/// Server errors that can occur during request processing.
#[derive(Error, Debug)]
pub enum ServerError {
    /// A tool execution failed with an error message.
    #[error("Tool error: {0}")]
    Tool(String),

    /// A requested resource was not found.
    #[error("Resource not found: {uri}")]
    ResourceNotFound { uri: String },

    /// A requested prompt was not found.
    #[error("Unknown prompt: {name}")]
    PromptNotFound { name: String },

    /// A required argument is missing.
    #[error("Missing required argument: {name}")]
    MissingArgument { name: String },

    /// An invalid argument was provided.
    #[error("Invalid argument: {0}")]
    InvalidArgument(String),

    /// Division by zero attempted.
    #[error("Division by zero")]
    DivisionByZero,

    /// JSON parsing or serialization failed.
    #[error("JSON error: {0}")]
    Json(String),

    /// Base64 encoding/decoding failed.
    #[error("Base64 error: {0}")]
    Base64(String),

    /// UTF-8 conversion failed.
    #[error("UTF-8 error: {0}")]
    Utf8(String),

    /// Resource does not support subscriptions.
    #[error("Resource does not support subscriptions: {uri}")]
    SubscriptionNotSupported { uri: String },
}

impl ServerError {
    /// Create a tool error from any displayable error.
    pub fn tool(msg: impl std::fmt::Display) -> Self {
        Self::Tool(msg.to_string())
    }

    /// Create a JSON error from any displayable error.
    pub fn json(err: impl std::fmt::Display) -> Self {
        Self::Json(err.to_string())
    }

    /// Create a Base64 error from any displayable error.
    pub fn base64(err: impl std::fmt::Display) -> Self {
        Self::Base64(err.to_string())
    }

    /// Create a UTF-8 error from any displayable error.
    pub fn utf8(err: impl std::fmt::Display) -> Self {
        Self::Utf8(err.to_string())
    }
}

impl From<ServerError> for rmcp::ErrorData {
    fn from(err: ServerError) -> Self {
        match &err {
            ServerError::ResourceNotFound { .. }
            | ServerError::PromptNotFound { .. }
            | ServerError::SubscriptionNotSupported { .. } => {
                Self::invalid_request(err.to_string(), None)
            }
            ServerError::MissingArgument { .. } | ServerError::InvalidArgument(_) => {
                Self::invalid_params(err.to_string(), None)
            }
            _ => Self::internal_error(err.to_string(), None),
        }
    }
}

impl From<serde_json::Error> for ServerError {
    fn from(err: serde_json::Error) -> Self {
        Self::Json(err.to_string())
    }
}

impl From<base64::DecodeError> for ServerError {
    fn from(err: base64::DecodeError) -> Self {
        Self::Base64(err.to_string())
    }
}

impl From<std::string::FromUtf8Error> for ServerError {
    fn from(err: std::string::FromUtf8Error) -> Self {
        Self::Utf8(err.to_string())
    }
}

/// A specialized Result type for server operations.
pub type Result<T> = std::result::Result<T, ServerError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_error_display() {
        let err = ServerError::Tool("something went wrong".to_string());
        assert_eq!(err.to_string(), "Tool error: something went wrong");
    }

    #[test]
    fn test_resource_not_found_display() {
        let err = ServerError::ResourceNotFound {
            uri: "test://missing".to_string(),
        };
        assert_eq!(err.to_string(), "Resource not found: test://missing");
    }

    #[test]
    fn test_prompt_not_found_display() {
        let err = ServerError::PromptNotFound {
            name: "unknown".to_string(),
        };
        assert_eq!(err.to_string(), "Unknown prompt: unknown");
    }

    #[test]
    fn test_missing_argument_display() {
        let err = ServerError::MissingArgument {
            name: "text".to_string(),
        };
        assert_eq!(err.to_string(), "Missing required argument: text");
    }

    #[test]
    fn test_division_by_zero_display() {
        let err = ServerError::DivisionByZero;
        assert_eq!(err.to_string(), "Division by zero");
    }

    #[test]
    fn test_error_to_mcp_error_data() {
        let err = ServerError::ResourceNotFound {
            uri: "test://x".to_string(),
        };
        let mcp_err: rmcp::ErrorData = err.into();
        assert!(mcp_err.message.contains("Resource not found"));
    }

    #[test]
    fn test_json_error_conversion() {
        let json_err = serde_json::from_str::<serde_json::Value>("invalid").unwrap_err();
        let err: ServerError = json_err.into();
        assert!(matches!(err, ServerError::Json(_)));
    }

    #[test]
    fn test_helper_constructors() {
        let err = ServerError::tool("test error");
        assert!(matches!(err, ServerError::Tool(_)));

        let err = ServerError::json("parse failed");
        assert!(matches!(err, ServerError::Json(_)));
    }
}
