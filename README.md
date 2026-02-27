# MCP Test Server

A comprehensive Model Context Protocol (MCP) test server written in Rust. Built on the [rmcp](https://crates.io/crates/rmcp) SDK, this server provides a complete implementation for testing MCP client libraries.

[![CI](https://github.com/nazq/test_mcp_servers/actions/workflows/ci.yml/badge.svg)](https://github.com/nazq/test_mcp_servers/actions/workflows/ci.yml)
[![codecov](https://codecov.io/gh/nazq/test_mcp_servers/graph/badge.svg)](https://codecov.io/gh/nazq/test_mcp_servers)
[![License](https://img.shields.io/badge/license-Apache--2.0-green.svg)](LICENSE)

## Purpose

**This server is designed for MCP client developers** who need to thoroughly test their client implementations. It provides a complete, spec-compliant MCP server that exercises all protocol features—tools, resources, prompts, completions, and both transport types.

Use cases:
- **Integration testing** with [Testcontainers](https://testcontainers.com/) or similar container orchestration
- **CI/CD pipelines** for MCP client libraries
- **Local development** when building MCP clients
- **MCP Apps testing** — verify your host renders interactive tool UIs via `_meta.ui.resourceUri`
- **Protocol compliance verification** against the MCP 2025-11-25 specification

The server is containerized and designed to be ephemeral—spin it up, run your tests, tear it down.

## Features

- **Full MCP 2025-11-25 specification compliance**
- **Streamable HTTP transport** (`/mcp` endpoint)
- **OAuth 2.1 mock endpoints** — full RFC 9728/8414/7591 discovery, DCR, PKCE, and token exchange
- **MCP Tasks support** — async long-running operations with cancellation
- **[MCP Apps](https://modelcontextprotocol.io/docs/extensions/apps) support** — 7 interactive UI tools with `_meta.ui.resourceUri`, served via `resources/read`
- **API key authentication** with constant-time comparison
- **33 tools** for comprehensive testing (math, string, encoding, utility, testing, tasks, MCP Apps)
- **14 resources** (static, dynamic, and `ui://` app resources) with subscription support
- **5 prompts** with argument validation
- **Auto-completion** for prompt arguments and resource URIs
- **Logging level control** via MCP protocol

## Quick Start

### Using Docker

```bash
docker run -p 3000:3000 ghcr.io/nazq/mcp-test-server:latest
```

### From Source

```bash
git clone https://github.com/nazq/test_mcp_servers
cd test_mcp_servers
cargo run --release
```

## Configuration

All configuration is done via environment variables:

| Variable | Default | Description |
|----------|---------|-------------|
| `MCP_HOST` | `0.0.0.0` | Server bind address |
| `MCP_PORT` | `3000` | Server listen port |
| `MCP_API_KEY` | (none) | If set, requires `Authorization: Bearer <key>` header |
| `MCP_LOG_LEVEL` | `info` | Logging level: `trace`, `debug`, `info`, `warn`, `error` |

## Endpoints

### Streamable HTTP Transport
- `GET /mcp` - Open SSE stream for server-initiated messages
- `POST /mcp` - Send JSON-RPC request, receive SSE stream or `202 Accepted`
- `DELETE /mcp` - Close session

### Health Check
- `GET /health` - Returns `{"status": "ok"}` (no authentication required)

### OAuth 2.1 Mock Endpoints
- `GET /.well-known/oauth-protected-resource` - RFC 9728 protected resource metadata
- `GET /.well-known/oauth-authorization-server` - RFC 8414 authorization server metadata
- `POST /oauth/register` - RFC 7591 dynamic client registration
- `GET /oauth/authorize` - Authorization endpoint (auto-approves for testing)
- `POST /oauth/token` - Token endpoint (authorization_code + refresh_token grants)

## Tools

The server provides 33 tools organized by category:

### Math Tools
| Tool | Description |
|------|-------------|
| `add` | Add two numbers |
| `subtract` | Subtract second number from first |
| `multiply` | Multiply two numbers |
| `divide` | Divide first by second (with zero check) |

### String Tools
| Tool | Description |
|------|-------------|
| `echo` | Return input text unchanged |
| `concat` | Concatenate multiple strings |
| `uppercase` | Convert to uppercase |
| `lowercase` | Convert to lowercase |
| `reverse` | Reverse a string |
| `length` | Get string length |

### Encoding Tools
| Tool | Description |
|------|-------------|
| `json_parse` | Parse JSON string |
| `json_stringify` | Convert value to JSON string |
| `base64_encode` | Base64 encode text |
| `base64_decode` | Base64 decode text |
| `hash_sha256` | SHA-256 hash of text |

### Utility Tools
| Tool | Description |
|------|-------------|
| `random_number` | Random number in range [min, max] |
| `random_uuid` | Generate UUID v4 |
| `current_time` | Current UTC timestamp (RFC3339) |

### Testing Tools
| Tool | Description |
|------|-------------|
| `sleep` | Sleep for N milliseconds |
| `fail` | Always returns an error |
| `fail_with_message` | Returns error with custom message |
| `slow_echo` | Echo with configurable delay |
| `nested_data` | Generate deeply nested JSON |
| `large_response` | Generate large text payload |
| `binary_data` | Generate random binary data (base64) |
| `noop` | No-op tool that returns immediately |

### Task Tools (MCP Tasks)

These tools support the [MCP Tasks](https://modelcontextprotocol.io/specification/2025-11-25/server/tasks) extension for async long-running operations. When called via `enqueue_task`, they run in the background and clients poll for status and results.

| Tool | Description |
|------|-------------|
| `task_slow_compute` | Simulate a slow computation (default: 5s), supports cancellation |
| `task_cancellable` | Long-running cancellable operation (default: 30s) |
| `task_fail` | Task that fails after a delay (default: 2s) with custom error |

### MCP App Tools

These tools implement the [MCP Apps extension](https://modelcontextprotocol.io/docs/extensions/apps). Each declares `_meta.ui.resourceUri` on the tool description, telling compatible hosts (VS Code Insiders, Claude Desktop) to fetch interactive HTML via `resources/read` and render it in a sandboxed iframe. The tool result is plain text — the UI loads independently.

#### Basic UI Tools
| Tool | `resourceUri` | Description |
|------|---------------|-------------|
| `ui_resource_button` | `ui://button/app.html` | Single button that calls the `echo` tool via JSON-RPC bridge |
| `ui_resource_form` | `ui://form/app.html` | Form that calls the `concat` tool via JSON-RPC bridge |
| `ui_resource_carousel` | `ui://carousel/app.html` | 3-card carousel, each card calls `echo` via JSON-RPC bridge |
| `ui_internal_only` | `ui://internal_only/app.html` | App-only tool (hidden from LLM, tests visibility filtering) |

#### Rich UI Tools
| Tool | `resourceUri` | Description |
|------|---------------|-------------|
| `ui_resource_dashboard` | `ui://dashboard/app.html` | Chart.js dashboard with tool call metrics and live data visualization |
| `ui_resource_data_table` | `ui://data_table/app.html` | Tabulator.js data table with filtering, sorting, and vanilla fallback |
| `ui_resource_pipeline` | `ui://pipeline/app.html` | Interactive ETL pipeline visualizer with stage-by-stage execution |

The HTML templates include a JSON-RPC shim (`mcp-app-shim.js`) that handles `window.postMessage` ↔ host bridging per the MCP Apps spec. Rich UI tools load CDN libraries (Chart.js, Tabulator) with graceful fallback to vanilla HTML/JS when CSP blocks them.

## Resources

### Static Resources
| URI | Type | Description |
|-----|------|-------------|
| `test://static/hello.txt` | text/plain | Simple text content |
| `test://static/data.json` | application/json | JSON data |
| `test://static/image.png` | image/png | Base64-encoded PNG |
| `test://static/large.txt` | text/plain | Large file (>10KB) |

### MCP App Resources
| URI | Type | Description |
|-----|------|-------------|
| `ui://button/app.html` | text/html;profile=mcp-app | Interactive button app |
| `ui://form/app.html` | text/html;profile=mcp-app | Interactive form app |
| `ui://carousel/app.html` | text/html;profile=mcp-app | Interactive 3-card carousel app |
| `ui://internal_only/app.html` | text/html;profile=mcp-app | Internal-only app (visibility test) |
| `ui://dashboard/app.html` | text/html;profile=mcp-app | Chart.js dashboard with metrics |
| `ui://data_table/app.html` | text/html;profile=mcp-app | Tabulator.js data table |
| `ui://pipeline/app.html` | text/html;profile=mcp-app | ETL pipeline visualizer |

### Dynamic Resources
| URI | Type | Description |
|-----|------|-------------|
| `test://dynamic/counter` | text/plain | Increments on each read |
| `test://dynamic/timestamp` | text/plain | Current timestamp |
| `test://dynamic/random` | text/plain | Random data (subscribable) |

### Resource Templates
| Template | Description |
|----------|-------------|
| `test://files/{path}` | Parameterized file access |

## Prompts

| Prompt | Arguments | Description |
|--------|-----------|-------------|
| `greeting` | `name` (required) | Simple greeting message |
| `code_review` | `code`, `language` (required) | Code review prompt with user/assistant roles |
| `summarize` | `text` (required) | Text summarization prompt |
| `translate` | `text`, `language` (required) | Translation prompt |
| `with_resource` | (none) | Prompt referencing embedded resource |

## Auto-Completion

The server provides completions for prompt arguments:

- `greeting.name`: Alice, Bob, Charlie, World
- `code_review.language`: rust, python, javascript, typescript, go
- `translate.language`: Spanish, French, German, Japanese, Chinese

And for resource templates:
- `test://files/{path}`: example.txt, data.json, config.yaml

## OAuth 2.1 Mock

The server includes a complete OAuth 2.1 mock implementation for testing MCP client authentication flows. All endpoints are served alongside the MCP server — no external identity provider needed.

**Flow:**

1. **Discovery** — Client fetches `/.well-known/oauth-protected-resource` (RFC 9728) to find the authorization server, then `/.well-known/oauth-authorization-server` (RFC 8414) for endpoint URLs
2. **Registration** — Client registers via `POST /oauth/register` (RFC 7591 DCR), receives a `client_id`
3. **Authorization** — Client redirects to `/oauth/authorize` with PKCE challenge; the mock auto-approves and redirects back with an authorization code
4. **Token Exchange** — Client exchanges the code at `POST /oauth/token` for an access token + refresh token
5. **API Access** — Client uses `Authorization: Bearer <token>` on `/mcp`

All tokens are test-only opaque strings (no JWT). PKCE is supported but verification is simplified for testing.

## Security

### API Key Authentication

When `MCP_API_KEY` is set, all endpoints except `/health` require authentication:

```bash
curl -H "Authorization: Bearer your-api-key" http://localhost:3000/mcp
```

### Origin Validation

The server validates `Origin` headers to prevent DNS rebinding attacks. Allowed origins:
- `http://localhost:*`
- `http://127.0.0.1:*`
- `https://localhost:*`
- Requests without `Origin` header (non-browser clients)

## Development

### Requirements

- Rust 1.88+ (edition 2024)
- Docker (optional, for containerized builds)

### Building

```bash
cargo build --release
```

### Testing

```bash
# Run all tests
cargo test

# Run with coverage
cargo llvm-cov --html
```

### Linting

```bash
cargo clippy -- -D warnings
cargo fmt --check
```

## Docker

### Build

```bash
docker build -t mcp-test-server .
```

### Run

```bash
docker run -p 3000:3000 \
  -e MCP_API_KEY=secret \
  -e MCP_LOG_LEVEL=debug \
  mcp-test-server
```

## License

MIT License - see [LICENSE](LICENSE) for details.

## Contributing

Contributions welcome! Please ensure:
1. All tests pass (`cargo test`)
2. No clippy warnings (`cargo clippy -- -D warnings`)
3. Code is formatted (`cargo fmt`)
4. Coverage doesn't drop below 85%

## Transport Support

This server uses **Streamable HTTP** transport only. SSE transport was deprecated in the
MCP specification and removed from the rmcp SDK:

- [rmcp PR #561](https://github.com/modelcontextprotocol/rust-sdk/pull/561) - SSE transport deprecated
- [rmcp PR #562](https://github.com/modelcontextprotocol/rust-sdk/pull/562) - SSE transport removed

If you need SSE transport, use version 0.3.x or earlier of this server.

## References

- [MCP Specification 2025-11-25](https://modelcontextprotocol.io/specification/2025-11-25)
- [MCP Apps Extension](https://modelcontextprotocol.io/docs/extensions/apps)
- [rmcp crate](https://crates.io/crates/rmcp)
