# MCP Test Server

A full-featured Model Context Protocol (MCP) test server in Rust, built on the [rmcp](https://crates.io/crates/rmcp) SDK. Supports both SSE and Streamable HTTP transports. Designed for integration testing of MCP clients and as a reference implementation.

## Goals

1. Full MCP 2025-11-25 specification compliance
2. Both SSE and Streamable HTTP transports (single server, both active)
3. API key authentication via `Authorization: Bearer <key>` header
4. Comprehensive test fixtures for tools, resources, and prompts
5. Lightweight Docker image for testcontainers integration
6. 90%+ test coverage via `cargo-llvm-cov`

## Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `MCP_API_KEY` | (none) | If set, requires `Authorization: Bearer <key>` header |
| `MCP_PORT` | `3000` | Server listen port |
| `MCP_HOST` | `0.0.0.0` | Server bind address |
| `MCP_LOG_LEVEL` | `info` | Logging level: `trace`, `debug`, `info`, `warn`, `error` |

## MCP Specification Requirements

### Lifecycle

- [ ] `initialize` request handling with protocol version negotiation
- [ ] `initialized` notification acknowledgment
- [ ] `ping` request/response
- [ ] Graceful shutdown on connection close

### Server Capabilities

```json
{
  "capabilities": {
    "prompts": { "listChanged": true },
    "resources": { "subscribe": true, "listChanged": true },
    "tools": { "listChanged": true },
    "logging": {},
    "completions": {}
  }
}
```

### Tools

- [ ] `tools/list` - Paginated list of available tools (cursor-based)
- [ ] `tools/call` - Execute tool with validated arguments
- [ ] `notifications/tools/list_changed` - Emit when tools change

**Test Tools (25+ for pagination testing):**

| Tool | Purpose |
|------|---------|
| `echo` | Returns input as output |
| `add` | Adds two numbers |
| `subtract` | Subtracts two numbers |
| `multiply` | Multiplies two numbers |
| `divide` | Divides two numbers (with zero check) |
| `concat` | Concatenates strings |
| `uppercase` | Converts string to uppercase |
| `lowercase` | Converts string to lowercase |
| `reverse` | Reverses a string |
| `length` | Returns string length |
| `json_parse` | Parses JSON string |
| `json_stringify` | Converts value to JSON string |
| `base64_encode` | Base64 encodes input |
| `base64_decode` | Base64 decodes input |
| `hash_sha256` | SHA-256 hash of input |
| `random_number` | Random number in range |
| `random_uuid` | Generates UUID v4 |
| `current_time` | Returns current UTC timestamp |
| `sleep` | Sleeps for N milliseconds (timeout test) |
| `fail` | Always returns error |
| `fail_with_message` | Returns error with custom message |
| `slow_echo` | Echo with configurable delay |
| `nested_data` | Returns deeply nested JSON |
| `large_response` | Returns large text payload |
| `binary_data` | Returns base64 binary blob |

### Resources

- [ ] `resources/list` - Paginated list of resources
- [ ] `resources/read` - Read resource content (text and blob)
- [ ] `resources/templates/list` - Return URI templates
- [ ] `resources/subscribe` / `resources/unsubscribe` - Subscription management
- [ ] `notifications/resources/updated` - Emit on subscribed resource change
- [ ] `notifications/resources/list_changed` - Emit when list changes

**Test Resources:**

| URI | Type | Purpose |
|-----|------|---------|
| `test://static/hello.txt` | text | Simple text content |
| `test://static/data.json` | text | JSON data |
| `test://static/image.png` | blob | Base64 binary |
| `test://static/large.txt` | text | Large file (pagination test) |
| `test://dynamic/counter` | text | Increments on each read |
| `test://dynamic/timestamp` | text | Current timestamp |
| `test://dynamic/random` | text | Random data (subscription test) |
| Template: `test://files/{path}` | - | Parameterized resource |

### Prompts

- [ ] `prompts/list` - Return available prompts
- [ ] `prompts/get` - Get prompt with resolved arguments
- [ ] `notifications/prompts/list_changed` - Emit when prompts change

**Test Prompts:**

| Prompt | Purpose |
|--------|---------|
| `greeting` | Simple "Hello, {name}!" |
| `code_review` | Multi-message with user/assistant roles |
| `summarize` | Takes text, returns summary prompt |
| `translate` | Translate {text} to {language} |
| `with_resource` | Prompt referencing embedded resource |

### Utilities

- [ ] `completion/complete` - Argument auto-completion
- [ ] `logging/setLevel` - Adjust server log verbosity
- [ ] Progress reporting for long operations
- [ ] Cancellation support via `$/cancelRequest`

## Transport Implementation

Both transports are served from a single server instance.

### SSE Transport

Endpoints:
- `GET /sse` - Opens SSE stream for server→client messages
- `POST /message` - Client→server JSON-RPC messages
- Session via `Mcp-Session-Id` header

### Streamable HTTP Transport

Single endpoint at `/mcp`:
- `POST /mcp` - Send JSON-RPC, returns SSE stream or `202 Accepted`
- `GET /mcp` - Open SSE stream for server-initiated messages
- Headers: `Accept: application/json, text/event-stream`, `Mcp-Protocol-Version: 2025-11-25`

### Security

- [ ] Validate `Origin` header (DNS rebinding protection)
- [ ] API key validation via `Authorization: Bearer <key>`
- [ ] Return `403 Forbidden` for auth failures
- [ ] Localhost binding when `MCP_HOST=127.0.0.1`

## Project Structure

```
mcp-test-server/
├── Cargo.toml
├── Dockerfile
├── CHANGELOG.md
├── src/
│   ├── main.rs                 # Entry point, config, transport setup
│   ├── lib.rs                  # Library exports
│   ├── config.rs               # Environment configuration
│   ├── server.rs               # ServerHandler implementation
│   ├── auth.rs                 # API key middleware
│   ├── tools/
│   │   ├── mod.rs              # Tool router aggregation
│   │   ├── math.rs             # Math operations
│   │   ├── string.rs           # String operations
│   │   ├── encoding.rs         # Base64, JSON, hashing
│   │   ├── utility.rs          # Time, UUID, random
│   │   └── testing.rs          # Error, delay, large response tools
│   ├── resources/
│   │   ├── mod.rs              # Resource handler
│   │   ├── static_resources.rs # Fixed content resources
│   │   └── dynamic_resources.rs# Changing content resources
│   └── prompts/
│       ├── mod.rs              # Prompt router
│       └── templates.rs        # Prompt definitions
└── tests/
    ├── common/
    │   └── mod.rs              # Test utilities
    ├── lifecycle_test.rs       # Initialize, ping, shutdown
    ├── tools_test.rs           # All tool operations
    ├── resources_test.rs       # Resource operations
    ├── prompts_test.rs         # Prompt operations
    ├── pagination_test.rs      # Cursor pagination
    ├── auth_test.rs            # API key validation
    └── transport_test.rs       # SSE and Streamable HTTP
```

## Dependencies

```toml
[package]
name = "mcp-test-server"
version = "0.1.0"
edition = "2024"
license = "MIT"
description = "MCP test server for integration testing"
repository = "https://github.com/YOUR_ORG/mcp-test-server"
keywords = ["mcp", "model-context-protocol", "testing"]
categories = ["development-tools::testing"]

[dependencies]
rmcp = { version = "0.10", features = [
    "server",
    "macros",
    "transport-sse-server",
    "transport-streamable-http-server",
] }

axum = "0.8"
tokio = { version = "1", features = ["full"] }
tower = "0.5"
tower-http = { version = "0.6", features = ["cors", "trace"] }

serde = { version = "1", features = ["derive"] }
serde_json = "1"
schemars = "1"

tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }

thiserror = "2"
uuid = { version = "1", features = ["v4"] }
base64 = "0.22"
sha2 = "0.10"
rand = "0.8"
chrono = { version = "0.4", features = ["serde"] }

[dev-dependencies]
reqwest = { version = "0.12", features = ["json"] }
tokio-test = "0.4"
eventsource-client = "0.13"
```

## Dockerfile

```dockerfile
FROM rust:1.83-alpine AS builder
WORKDIR /app
RUN apk add --no-cache musl-dev
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --release

FROM alpine:3.21
RUN apk add --no-cache ca-certificates
COPY --from=builder /app/target/release/mcp-test-server /usr/local/bin/
ENV MCP_PORT=3000 MCP_HOST=0.0.0.0
EXPOSE 3000
ENTRYPOINT ["mcp-test-server"]
```

Target image size: < 20MB

## CI/CD

### GitHub Actions Workflows

**PR Checks** (`.github/workflows/ci.yml`):
- Runs on all PRs to `main`
- `cargo fmt --check`
- `cargo clippy -- -D warnings`
- `cargo test`
- `cargo llvm-cov --fail-under 90`
- Docker build test

**Release** (`.github/workflows/release.yml`):
- Triggered by release-plz merge to `main`
- Uses [release-plz](https://release-plz.ieni.dev/) for changelog and versioning
- Publishes to crates.io
- Builds multi-arch Docker images (amd64, arm64)
- Pushes to `ghcr.io/YOUR_ORG/mcp-test-server`

### Release Flow

1. PRs to `main` trigger CI checks
2. Merge to `main` triggers release-plz to create release PR
3. Release PR contains updated `CHANGELOG.md` and version bump
4. Accepting release PR triggers:
   - Git tag creation
   - GitHub Release with notes
   - crates.io publish
   - Docker image build and push to GHCR

## Testing Strategy

### Unit Tests
- Tool input validation
- Resource content generation
- Prompt argument substitution
- Config parsing

### Integration Tests
- Full lifecycle: initialize → operations → shutdown
- Both transport modes
- API key enforcement
- Pagination with cursors
- Resource subscriptions
- Error handling paths

### Coverage Target
- Minimum 90% line coverage
- Measured via `cargo llvm-cov`
- CI fails if coverage drops below threshold

## Success Criteria

1. All MCP 2025-11-25 spec methods implemented
2. Both SSE and Streamable HTTP transports functional
3. API key authentication working
4. Docker image < 20MB
5. 90%+ test coverage
6. Clean release-plz workflow
7. Published to crates.io and GHCR

## Out of Scope

- stdio transport (not needed for HTTP testing)
- Sampling capability (client-side feature)
- Elicitation capability (client-side feature)
- Custom tool registration API (fixtures are static)
- Persistent storage (in-memory only)
- Rate limiting

## References

- [MCP Specification 2025-11-25](https://modelcontextprotocol.io/specification/2025-11-25)
- [rmcp crate](https://crates.io/crates/rmcp)
- [release-plz](https://release-plz.ieni.dev/)
