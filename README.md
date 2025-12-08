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
- **Protocol compliance verification** against the MCP 2025-11-25 specification

The server is containerized and designed to be ephemeral—spin it up, run your tests, tear it down.

## Features

- **Full MCP 2025-11-25 specification compliance**
- **Dual transport support**: SSE and Streamable HTTP on a single server
- **API key authentication** with constant-time comparison
- **25 tools** for comprehensive testing (math, string, encoding, utility, testing)
- **8 resources** (static and dynamic) with subscription support
- **5 prompts** with argument validation
- **Auto-completion** for prompt arguments and resource URIs
- **Logging level control** via MCP protocol

## Quick Start

### Using Docker

```bash
docker run -p 3000:3000 ghcr.io/nazq/test_mcp_servers:latest
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

### SSE Transport
- `GET /sse` - Opens SSE stream for server-to-client messages
- `POST /message` - Client-to-server JSON-RPC messages

### Streamable HTTP Transport
- `GET /mcp` - Open SSE stream for server-initiated messages
- `POST /mcp` - Send JSON-RPC request, receive SSE stream or `202 Accepted`
- `DELETE /mcp` - Close session

### Health Check
- `GET /health` - Returns `{"status": "ok"}` (no authentication required)

## Tools

The server provides 25 tools organized by category:

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

## Resources

### Static Resources
| URI | Type | Description |
|-----|------|-------------|
| `test://static/hello.txt` | text/plain | Simple text content |
| `test://static/data.json` | application/json | JSON data |
| `test://static/image.png` | image/png | Base64-encoded PNG |
| `test://static/large.txt` | text/plain | Large file (>10KB) |

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

- Rust 1.85+ (edition 2024)
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

## References

- [MCP Specification 2025-11-25](https://modelcontextprotocol.io/specification/2025-11-25)
- [rmcp crate](https://crates.io/crates/rmcp)
