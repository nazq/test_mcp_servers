//! Authentication middleware for API key validation and origin checking.
//!
//! # Example Usage
//!
//! ```rust,no_run
//! use axum::{routing::get, Router, middleware};
//! use mcp_test_server::{Config, auth::auth_middleware};
//!
//! async fn health_handler() -> &'static str {
//!     "OK"
//! }
//!
//! async fn protected_handler() -> &'static str {
//!     "Protected resource"
//! }
//!
//! # async fn example() {
//! let config = Config::default();
//!
//! // Create protected routes with auth middleware
//! let protected_routes: Router = Router::new()
//!     .route("/sse", get(protected_handler))
//!     .route("/message", get(protected_handler))
//!     .route("/mcp", get(protected_handler))
//!     .layer(middleware::from_fn_with_state(config.clone(), auth_middleware));
//!
//! // Combine with public routes
//! let app: Router = Router::new()
//!     .route("/health", get(health_handler))  // No auth
//!     .merge(protected_routes);
//! # }
//! ```

use axum::{
    body::Body,
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use subtle::ConstantTimeEq;

use crate::config::Config;

/// Error response for authentication failures.
#[derive(Debug, Serialize)]
pub struct AuthError {
    error: String,
    message: String,
}

impl AuthError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            error: "forbidden".to_string(),
            message: message.into(),
        }
    }
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let body = serde_json::to_string(&self).unwrap_or_else(|_| {
            r#"{"error":"forbidden","message":"Authentication failed"}"#.to_string()
        });
        (StatusCode::FORBIDDEN, body).into_response()
    }
}

/// Authentication middleware that validates API keys and origins.
///
/// This middleware performs two security checks:
/// 1. API key validation (if configured)
/// 2. Origin header validation (DNS rebinding protection)
///
/// # API Key Validation
///
/// If `MCP_API_KEY` is set in the configuration, this middleware checks the
/// `Authorization` header for a bearer token. The comparison is done in
/// constant time to prevent timing attacks.
///
/// # Origin Validation
///
/// Validates the `Origin` header to prevent DNS rebinding attacks. Allows:
/// - No Origin header (non-browser clients)
/// - `http://localhost:*`
/// - `http://127.0.0.1:*`
/// - `https://localhost:*`
///
/// # Errors
///
/// Returns `403 Forbidden` with a JSON error body if:
/// - API key is required but missing
/// - API key is invalid
/// - Origin is present but not allowed
#[allow(clippy::cognitive_complexity)]
pub async fn auth_middleware(
    State(config): State<Config>,
    request: Request<Body>,
    next: Next,
) -> Result<Response, AuthError> {
    // Validate API key if configured
    if let Some(ref expected_key) = config.api_key {
        let auth_header = request
            .headers()
            .get("authorization")
            .and_then(|v| v.to_str().ok());

        match auth_header {
            Some(header) if header.starts_with("Bearer ") => {
                let token = &header[7..];

                // Constant-time comparison to prevent timing attacks
                if !constant_time_compare(token.as_bytes(), expected_key.as_bytes()) {
                    tracing::debug!("API key validation failed");
                    return Err(AuthError::new("Invalid API key"));
                }
            }
            Some(_) => {
                tracing::debug!("Invalid Authorization header format");
                return Err(AuthError::new(
                    "Invalid Authorization header format. Expected: Bearer <token>",
                ));
            }
            None => {
                tracing::debug!("Missing Authorization header");
                return Err(AuthError::new("Missing Authorization header"));
            }
        }
    }

    // Validate origin header for DNS rebinding protection
    if let Some(origin) = request
        .headers()
        .get("origin")
        .and_then(|v| v.to_str().ok())
        && !is_allowed_origin(origin)
    {
        tracing::debug!(origin = %origin, "Origin not allowed");
        return Err(AuthError::new("Origin not allowed"));
    }

    Ok(next.run(request).await)
}

/// Compare two byte slices in constant time.
///
/// This prevents timing attacks when comparing API keys.
fn constant_time_compare(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    a.ct_eq(b).into()
}

/// Check if an origin is allowed.
///
/// Allows localhost origins to prevent DNS rebinding attacks while still
/// permitting local development.
fn is_allowed_origin(origin: &str) -> bool {
    // Allow localhost origins (any port)
    origin.starts_with("http://localhost")
        || origin.starts_with("http://127.0.0.1")
        || origin.starts_with("https://localhost")
        // Allow VS Code / Electron origins
        || origin.starts_with("vscode-file://")
        || origin.starts_with("vscode-webview://")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constant_time_compare() {
        assert!(constant_time_compare(b"secret", b"secret"));
        assert!(!constant_time_compare(b"secret", b"wrong"));
        assert!(!constant_time_compare(b"short", b"longer"));
    }

    #[test]
    fn test_allowed_origins() {
        assert!(is_allowed_origin("http://localhost:3000"));
        assert!(is_allowed_origin("http://127.0.0.1:8080"));
        assert!(is_allowed_origin("https://localhost"));
        assert!(is_allowed_origin("vscode-file://vscode-app"));
        assert!(is_allowed_origin("vscode-webview://abc123"));
        assert!(!is_allowed_origin("http://evil.com"));
        assert!(!is_allowed_origin("https://example.com"));
    }

    #[test]
    fn test_auth_error_serialization() {
        let error = AuthError::new("Test message");
        let json = serde_json::to_string(&error).unwrap();
        assert!(json.contains("forbidden"));
        assert!(json.contains("Test message"));
    }
}
