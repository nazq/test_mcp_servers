# Authentication Middleware Implementation

This document describes the authentication middleware implementation for the MCP test server.

## Overview

The authentication middleware (`src/auth.rs`) provides two security features:

1. **API Key Authentication** - Bearer token validation
2. **Origin Validation** - DNS rebinding protection

## Features

### API Key Authentication

- Checks `Authorization: Bearer <key>` header
- Compares against `MCP_API_KEY` environment variable (via Config)
- Uses constant-time comparison to prevent timing attacks (via `subtle` crate)
- Returns `403 Forbidden` if key is missing or invalid
- Skips authentication if `MCP_API_KEY` is not configured

### Origin Validation

- Validates the `Origin` header to prevent DNS rebinding attacks
- Allows the following origins:
  - No Origin header (non-browser clients like curl, SDKs)
  - `http://localhost:*` (any port)
  - `http://127.0.0.1:*` (any port)
  - `https://localhost:*` (any port)
- Returns `403 Forbidden` if origin is present but not allowed

## Implementation Details

### Dependencies Added

```toml
subtle = "2.6"  # For constant-time comparison
```

### Core Functions

#### `auth_middleware`

The main middleware function that validates both API keys and origins.

```rust
pub async fn auth_middleware(
    State(config): State<Config>,
    request: Request<Body>,
    next: Next,
) -> Result<Response, AuthError>
```

#### `constant_time_compare`

Prevents timing attacks when comparing API keys:

```rust
fn constant_time_compare(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    a.ct_eq(b).into()
}
```

#### `is_allowed_origin`

Validates origins for DNS rebinding protection:

```rust
fn is_allowed_origin(origin: &str) -> bool {
    origin.starts_with("http://localhost")
        || origin.starts_with("http://127.0.0.1")
        || origin.starts_with("https://localhost")
}
```

### Error Responses

All authentication failures return:
- HTTP Status: `403 Forbidden`
- Content-Type: `application/json`
- Body format:
  ```json
  {
    "error": "forbidden",
    "message": "<specific error message>"
  }
  ```

Error messages:
- `"Missing Authorization header"` - No Authorization header when API key is required
- `"Invalid Authorization header format. Expected: Bearer <token>"` - Wrong format
- `"Invalid API key"` - API key doesn't match
- `"Origin not allowed"` - Origin header present but not in allowed list

### Logging

Authentication failures are logged at `debug` level to avoid leaking keys:

```rust
tracing::debug!("API key validation failed");
tracing::debug!(origin = %origin, "Origin not allowed");
```

## Usage in Router

### Basic Setup

```rust
use axum::{routing::get, Router, middleware};
use mcp_test_server::{Config, auth::auth_middleware};

let config = Config::from_env();

// Create protected routes
let protected_routes = Router::new()
    .route("/sse", get(sse_handler))
    .route("/message", post(message_handler))
    .route("/mcp", get(mcp_handler).post(mcp_handler))
    .layer(middleware::from_fn_with_state(config.clone(), auth_middleware));

// Combine with public routes
let app = Router::new()
    .route("/health", get(health_handler))  // No authentication
    .merge(protected_routes);
```

### Configuration

Set environment variable to enable authentication:

```bash
# Enable API key authentication
export MCP_API_KEY="your-secret-key-here"

# Start server
cargo run
```

### Testing with curl

```bash
# Without API key (when MCP_API_KEY is set) - FORBIDDEN
curl http://localhost:3000/sse

# With correct API key - SUCCESS
curl -H "Authorization: Bearer your-secret-key-here" http://localhost:3000/sse

# With wrong API key - FORBIDDEN
curl -H "Authorization: Bearer wrong-key" http://localhost:3000/sse

# Health endpoint (no auth required) - SUCCESS
curl http://localhost:3000/health

# With disallowed origin - FORBIDDEN
curl -H "Origin: http://evil.com" http://localhost:3000/sse

# With allowed origin - SUCCESS
curl -H "Origin: http://localhost:3000" http://localhost:3000/sse
```

## Security Considerations

1. **Constant-Time Comparison**: Uses `subtle::ConstantTimeEq` to prevent timing attacks
2. **Debug Logging Only**: Authentication failures are logged at debug level, not info/warn
3. **No Key Exposure**: Error messages never include the actual API key
4. **DNS Rebinding Protection**: Origin validation prevents DNS rebinding attacks
5. **Graceful Degradation**: When `MCP_API_KEY` is not set, authentication is skipped

## Testing

Unit tests in `src/auth.rs`:
- `test_constant_time_compare` - Verifies constant-time comparison
- `test_allowed_origins` - Validates origin checking logic
- `test_auth_error_serialization` - Tests error JSON serialization

Integration tests in `tests/auth_test.rs`:
- No auth configuration scenarios
- Missing/invalid/correct API key scenarios
- Origin validation scenarios
- Combined auth + origin scenarios

Run tests:
```bash
# Run all tests
cargo test

# Run only auth tests
cargo test auth

# Run with output
cargo test auth -- --nocapture
```

## Compliance

The implementation satisfies all requirements:

- ✅ API key validation via `Authorization: Bearer <key>`
- ✅ Comparison against `MCP_API_KEY` from Config
- ✅ Returns 403 Forbidden on auth failure
- ✅ Skips auth when `MCP_API_KEY` not configured
- ✅ Origin validation against allowed origins
- ✅ DNS rebinding attack prevention
- ✅ Localhost origins allowed by default
- ✅ Constant-time string comparison (subtle crate)
- ✅ Debug-level logging for auth failures
- ✅ JSON error responses
- ✅ Graceful handling of missing headers
- ✅ Compiles with `cargo check`
- ✅ Passes `cargo clippy -- -D warnings` (for auth.rs specifically)
