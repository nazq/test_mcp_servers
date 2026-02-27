//! Mock OAuth 2.1 endpoints for testing MCP client authentication flows.
//!
//! Implements a self-contained OAuth 2.1 mock server that requires no external
//! identity provider. Follows the MCP specification's OAuth flow:
//!
//! 1. Client discovers OAuth metadata via `.well-known` endpoints
//! 2. Client registers via `/oauth/register` (RFC 7591 DCR)
//! 3. Client redirects to `/oauth/authorize` with PKCE
//! 4. User "authorizes" (auto-approved for testing)
//! 5. Client exchanges code at `/oauth/token`
//! 6. Client uses Bearer token for `/mcp`
//!
//! All tokens are test tokens — no real cryptographic verification.

use std::collections::HashMap;
use std::fmt::Write;
use std::sync::Arc;

use axum::Router;
use axum::extract::{Query, State};
use axum::response::{Html, IntoResponse, Json, Redirect};
use axum::routing::{get, post};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

/// Shared state for the OAuth mock server.
#[derive(Debug, Clone)]
pub struct OAuthState {
    /// Base URL of this server (e.g., `http://localhost:3000`).
    pub issuer: String,
    /// Registered clients: `client_id` -> client metadata.
    clients: Arc<Mutex<HashMap<String, RegisteredClient>>>,
    /// Pending authorization codes: code -> grant metadata.
    codes: Arc<Mutex<HashMap<String, AuthorizationGrant>>>,
}

#[derive(Debug, Clone, Serialize)]
struct RegisteredClient {
    client_id: String,
    client_name: Option<String>,
    redirect_uris: Vec<String>,
}

#[derive(Debug, Clone)]
struct AuthorizationGrant {
    _client_id: String,
    redirect_uri: String,
    code_challenge: Option<String>,
    _code_challenge_method: Option<String>,
    scope: Option<String>,
}

impl OAuthState {
    /// Create a new OAuth state with the given issuer URL.
    pub fn new(issuer: impl Into<String>) -> Self {
        Self {
            issuer: issuer.into(),
            clients: Arc::new(Mutex::new(HashMap::new())),
            codes: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

/// Build the OAuth router with all discovery and flow endpoints.
///
/// All routes are public (no auth) — this is the OAuth provider itself.
pub fn oauth_router(state: OAuthState) -> Router {
    Router::new()
        .route(
            "/.well-known/oauth-protected-resource",
            get(protected_resource_metadata),
        )
        .route(
            "/.well-known/oauth-authorization-server",
            get(authorization_server_metadata),
        )
        .route("/oauth/register", post(register_client))
        .route("/oauth/authorize", get(authorize))
        .route("/oauth/token", post(token_exchange))
        .with_state(state)
}

// =============================================================================
// RFC 9728 — OAuth Protected Resource Metadata
// =============================================================================

/// `GET /.well-known/oauth-protected-resource`
///
/// RFC 9728: tells MCP clients where to find the authorization server.
async fn protected_resource_metadata(State(state): State<OAuthState>) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "resource": state.issuer,
        "authorization_servers": [state.issuer],
        "bearer_methods_supported": ["header"],
        "scopes_supported": ["mcp"]
    }))
}

// =============================================================================
// RFC 8414 — OAuth Authorization Server Metadata
// =============================================================================

/// `GET /.well-known/oauth-authorization-server`
///
/// RFC 8414: tells MCP clients the full OAuth endpoint layout.
async fn authorization_server_metadata(State(state): State<OAuthState>) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "issuer": state.issuer,
        "authorization_endpoint": format!("{}/oauth/authorize", state.issuer),
        "token_endpoint": format!("{}/oauth/token", state.issuer),
        "registration_endpoint": format!("{}/oauth/register", state.issuer),
        "response_types_supported": ["code"],
        "grant_types_supported": ["authorization_code", "refresh_token"],
        "code_challenge_methods_supported": ["S256", "plain"],
        "token_endpoint_auth_methods_supported": ["none"],
        "scopes_supported": ["mcp"],
        "service_documentation": "https://github.com/nazq/test_mcp_servers"
    }))
}

// =============================================================================
// RFC 7591 — Dynamic Client Registration
// =============================================================================

#[derive(Debug, Deserialize)]
struct RegisterRequest {
    #[serde(default)]
    client_name: Option<String>,
    #[serde(default)]
    redirect_uris: Vec<String>,
}

/// `POST /oauth/register`
///
/// RFC 7591: Dynamic client registration. Issues a new `client_id` for any
/// registrant. No authentication required (public DCR).
async fn register_client(
    State(state): State<OAuthState>,
    Json(request): Json<RegisterRequest>,
) -> Json<serde_json::Value> {
    let client_id = format!("test-client-{}", uuid::Uuid::new_v4());

    let client = RegisteredClient {
        client_id: client_id.clone(),
        client_name: request.client_name.clone(),
        redirect_uris: request.redirect_uris.clone(),
    };

    state.clients.lock().await.insert(client_id.clone(), client);

    Json(serde_json::json!({
        "client_id": client_id,
        "client_name": request.client_name,
        "redirect_uris": request.redirect_uris,
        "grant_types": ["authorization_code", "refresh_token"],
        "response_types": ["code"],
        "token_endpoint_auth_method": "none"
    }))
}

// =============================================================================
// Authorization Endpoint
// =============================================================================

#[derive(Debug, Deserialize)]
struct AuthorizeParams {
    client_id: String,
    redirect_uri: String,
    #[serde(default = "default_response_type")]
    response_type: String,
    #[serde(default)]
    state: Option<String>,
    #[serde(default)]
    code_challenge: Option<String>,
    #[serde(default)]
    code_challenge_method: Option<String>,
    #[serde(default)]
    scope: Option<String>,
}

fn default_response_type() -> String {
    "code".to_string()
}

/// `GET /oauth/authorize`
///
/// Authorization endpoint. In a real server this would show a consent page.
/// For testing, we auto-approve and redirect back with an authorization code.
async fn authorize(
    State(state): State<OAuthState>,
    Query(params): Query<AuthorizeParams>,
) -> impl IntoResponse {
    // Validate response_type
    if params.response_type != "code" {
        return Html(format!(
            "<h1>Error</h1><p>Unsupported response_type: {}</p>",
            params.response_type
        ))
        .into_response();
    }

    // Generate authorization code
    let code = format!("test-code-{}", uuid::Uuid::new_v4());

    // Store the grant
    let grant = AuthorizationGrant {
        _client_id: params.client_id,
        redirect_uri: params.redirect_uri.clone(),
        code_challenge: params.code_challenge,
        _code_challenge_method: params.code_challenge_method,
        scope: params.scope,
    };
    state.codes.lock().await.insert(code.clone(), grant);

    // Build redirect URL with code and state
    let mut redirect_url = params.redirect_uri;
    redirect_url.push_str(if redirect_url.contains('?') { "&" } else { "?" });
    let _ = write!(redirect_url, "code={code}");
    if let Some(ref s) = params.state {
        let _ = write!(redirect_url, "&state={s}");
    }

    Redirect::to(&redirect_url).into_response()
}

// =============================================================================
// Token Endpoint
// =============================================================================

#[derive(Debug, Deserialize)]
struct TokenRequest {
    grant_type: String,
    #[serde(default)]
    code: Option<String>,
    #[serde(default)]
    redirect_uri: Option<String>,
    #[serde(default)]
    _client_id: Option<String>,
    #[serde(default)]
    code_verifier: Option<String>,
    #[serde(default)]
    refresh_token: Option<String>,
}

/// `POST /oauth/token`
///
/// Token endpoint. Exchanges authorization codes for access tokens.
/// Also handles refresh token grants.
///
/// For testing, returns a simple opaque token — no JWT signing.
#[allow(clippy::significant_drop_tightening)]
async fn token_exchange(
    State(state): State<OAuthState>,
    axum::Form(request): axum::Form<TokenRequest>,
) -> impl IntoResponse {
    match request.grant_type.as_str() {
        "authorization_code" => {
            let Some(code) = &request.code else {
                return (
                    axum::http::StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({
                        "error": "invalid_request",
                        "error_description": "Missing code parameter"
                    })),
                )
                    .into_response();
            };

            // Look up and consume the authorization code
            let Some(grant) = state.codes.lock().await.remove(code) else {
                return (
                    axum::http::StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({
                        "error": "invalid_grant",
                        "error_description": "Invalid or expired authorization code"
                    })),
                )
                    .into_response();
            };

            // Validate redirect_uri matches
            if let Some(ref uri) = request.redirect_uri
                && *uri != grant.redirect_uri
            {
                return (
                    axum::http::StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({
                        "error": "invalid_grant",
                        "error_description": "redirect_uri mismatch"
                    })),
                )
                    .into_response();
            }

            // PKCE verification (simplified — accept any verifier for testing)
            if grant.code_challenge.is_some() && request.code_verifier.is_none() {
                return (
                    axum::http::StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({
                        "error": "invalid_grant",
                        "error_description": "Missing code_verifier for PKCE"
                    })),
                )
                    .into_response();
            }

            // Issue tokens
            let access_token = format!("test-access-{}", uuid::Uuid::new_v4());
            let refresh_token = format!("test-refresh-{}", uuid::Uuid::new_v4());

            Json(serde_json::json!({
                "access_token": access_token,
                "token_type": "Bearer",
                "expires_in": 3600,
                "refresh_token": refresh_token,
                "scope": grant.scope.unwrap_or_else(|| "mcp".to_string())
            }))
            .into_response()
        }

        "refresh_token" => {
            if request.refresh_token.is_none() {
                return (
                    axum::http::StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({
                        "error": "invalid_request",
                        "error_description": "Missing refresh_token"
                    })),
                )
                    .into_response();
            }

            // For testing, always issue a new token pair
            let access_token = format!("test-access-{}", uuid::Uuid::new_v4());
            let refresh_token = format!("test-refresh-{}", uuid::Uuid::new_v4());

            Json(serde_json::json!({
                "access_token": access_token,
                "token_type": "Bearer",
                "expires_in": 3600,
                "refresh_token": refresh_token,
                "scope": "mcp"
            }))
            .into_response()
        }

        _ => (
            axum::http::StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "unsupported_grant_type",
                "error_description": format!("Unsupported grant_type: {}", request.grant_type)
            })),
        )
            .into_response(),
    }
}

#[cfg(test)]
#[allow(clippy::significant_drop_tightening)]
mod tests {
    use super::*;

    fn test_state() -> OAuthState {
        OAuthState::new("http://localhost:3000")
    }

    #[tokio::test]
    async fn test_protected_resource_metadata() {
        let state = test_state();
        let result = protected_resource_metadata(State(state)).await;
        let json = result.0;
        assert_eq!(json["resource"], "http://localhost:3000");
        assert!(
            json["authorization_servers"]
                .as_array()
                .unwrap()
                .contains(&serde_json::json!("http://localhost:3000"))
        );
    }

    #[tokio::test]
    async fn test_authorization_server_metadata() {
        let state = test_state();
        let result = authorization_server_metadata(State(state)).await;
        let json = result.0;
        assert_eq!(json["issuer"], "http://localhost:3000");
        assert_eq!(
            json["authorization_endpoint"],
            "http://localhost:3000/oauth/authorize"
        );
        assert_eq!(json["token_endpoint"], "http://localhost:3000/oauth/token");
        assert_eq!(
            json["registration_endpoint"],
            "http://localhost:3000/oauth/register"
        );
        assert!(
            json["code_challenge_methods_supported"]
                .as_array()
                .unwrap()
                .contains(&serde_json::json!("S256"))
        );
    }

    #[tokio::test]
    async fn test_register_client() {
        let state = test_state();
        let request = RegisterRequest {
            client_name: Some("Test Client".to_string()),
            redirect_uris: vec!["http://localhost:8080/callback".to_string()],
        };
        let result = register_client(State(state.clone()), Json(request)).await;
        let json = result.0;

        assert!(
            json["client_id"]
                .as_str()
                .unwrap()
                .starts_with("test-client-")
        );
        assert_eq!(json["client_name"], "Test Client");
        assert_eq!(json["token_endpoint_auth_method"], "none");

        // Verify client was stored
        assert_eq!(state.clients.lock().await.len(), 1);
    }

    #[tokio::test]
    async fn test_full_oauth_flow() {
        let state = test_state();

        // 1. Register client
        let reg_request = RegisterRequest {
            client_name: Some("Flow Test".to_string()),
            redirect_uris: vec!["http://localhost:8080/callback".to_string()],
        };
        let reg_result = register_client(State(state.clone()), Json(reg_request)).await;
        let client_id = reg_result.0["client_id"].as_str().unwrap().to_string();

        // 2. Authorize — creates a code and redirects
        let auth_params = AuthorizeParams {
            client_id: client_id.clone(),
            redirect_uri: "http://localhost:8080/callback".to_string(),
            response_type: "code".to_string(),
            state: Some("test-state".to_string()),
            code_challenge: None,
            code_challenge_method: None,
            scope: Some("mcp".to_string()),
        };
        let auth_result = authorize(State(state.clone()), Query(auth_params)).await;
        let response = auth_result.into_response();

        // Should redirect (302/307)
        assert!(response.status().is_redirection());
        let location = response
            .headers()
            .get("location")
            .unwrap()
            .to_str()
            .unwrap();
        assert!(location.starts_with("http://localhost:8080/callback?"));
        assert!(location.contains("code=test-code-"));
        assert!(location.contains("state=test-state"));

        // Extract code from redirect URL
        let code = location
            .split("code=")
            .nth(1)
            .unwrap()
            .split('&')
            .next()
            .unwrap()
            .to_string();

        // 3. Exchange code for token
        let token_request = TokenRequest {
            grant_type: "authorization_code".to_string(),
            code: Some(code),
            redirect_uri: Some("http://localhost:8080/callback".to_string()),
            _client_id: Some(client_id),
            code_verifier: None,
            refresh_token: None,
        };
        let token_result = token_exchange(State(state.clone()), axum::Form(token_request)).await;
        let token_response = token_result.into_response();
        assert!(token_response.status().is_success());
    }

    #[tokio::test]
    async fn test_token_invalid_code() {
        let state = test_state();
        let request = TokenRequest {
            grant_type: "authorization_code".to_string(),
            code: Some("invalid-code".to_string()),
            redirect_uri: None,
            _client_id: None,
            code_verifier: None,
            refresh_token: None,
        };
        let result = token_exchange(State(state), axum::Form(request)).await;
        let response = result.into_response();
        assert_eq!(response.status(), axum::http::StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_refresh_token() {
        let state = test_state();
        let request = TokenRequest {
            grant_type: "refresh_token".to_string(),
            code: None,
            redirect_uri: None,
            _client_id: None,
            code_verifier: None,
            refresh_token: Some("test-refresh-token".to_string()),
        };
        let result = token_exchange(State(state), axum::Form(request)).await;
        let response = result.into_response();
        assert!(response.status().is_success());
    }

    #[tokio::test]
    async fn test_unsupported_grant_type() {
        let state = test_state();
        let request = TokenRequest {
            grant_type: "client_credentials".to_string(),
            code: None,
            redirect_uri: None,
            _client_id: None,
            code_verifier: None,
            refresh_token: None,
        };
        let result = token_exchange(State(state), axum::Form(request)).await;
        let response = result.into_response();
        assert_eq!(response.status(), axum::http::StatusCode::BAD_REQUEST);
    }
}
