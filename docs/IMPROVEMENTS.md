# MCP Test Server - Improvement Recommendations

Based on analysis against Rust development guidelines, this document outlines recommended improvements organized by priority and effort.

## Summary

| Category | Current State | Recommendations |
|----------|---------------|-----------------|
| Project Structure | Good | Add clippy.toml, examples |
| Error Handling | Mixed | Standardize with thiserror |
| Testing | Good (85%+) | Add benchmarks, integration test helpers |
| Performance | Good | Add release profile tuning |
| Documentation | Good | Add architecture docs |
| Type Safety | Excellent | Minor improvements |

---

## Code Coverage Analysis

**Overall Coverage: 85.13% lines** (1179/1385 lines covered)

### Coverage by File

| File | Lines | Covered | Coverage | Status |
|------|-------|---------|----------|--------|
| `src/auth.rs` | 80 | 77 | **96.25%** | Excellent |
| `src/config.rs` | 75 | 58 | **77.33%** | Needs improvement |
| `src/prompts/mod.rs` | 89 | 65 | **73.03%** | Needs improvement |
| `src/prompts/templates.rs` | 177 | 176 | **99.44%** | Excellent |
| `src/resources/dynamic_resources.rs` | 81 | 78 | **96.30%** | Excellent |
| `src/resources/mod.rs` | 94 | 84 | **89.36%** | Good |
| `src/resources/static_resources.rs` | 103 | 103 | **100.00%** | Perfect |
| `src/server.rs` | 686 | 538 | **78.43%** | Needs improvement |

### Test Statistics

- **Total tests**: 135 tests
- **Unit tests**: 47 tests
- **Integration tests**: 88 tests
- **All tests passing**

### Coverage Gaps Analysis

#### `src/config.rs` (77.33%)
- **Gap**: `from_env()` function not tested
- **Reason**: `env::set_var` is unsafe in edition 2024, forbidden by `unsafe_code = "forbid"`
- **Recommendation**: Document this limitation; consider integration tests with actual env vars

#### `src/prompts/mod.rs` (73.03%)
- **Gap**: `list_prompts_impl` and `get_prompt_impl` not exercised via unit tests
- **Reason**: Methods require `RequestContext<RoleServer>` which is internal to rmcp
- **Recommendation**: These are tested via integration tests; consider adding internal test helpers

#### `src/server.rs` (78.43%)
- **Gap**: `run()` method (~100 lines), `complete()` method branches, `set_level()` method
- **Reason**: Server lifecycle requires full HTTP stack; some branches require specific MCP client behavior
- **Recommendation**:
  - `run()` tested via lifecycle_test.rs integration tests
  - Add unit tests for `set_level()` logging levels
  - Add completion tests for remaining prompt/resource branches

### Coverage Improvement Opportunities

| Area | Current | Target | Effort | Recommendation |
|------|---------|--------|--------|----------------|
| `set_level()` | 0% | 100% | Low | Add unit test for each log level |
| `complete()` branches | ~60% | 90% | Medium | Add tests for resource completions |
| Config `from_env()` | 0% | N/A | N/A | Document as intentionally untested |

### Recommended Test Additions

```rust
// Test for set_level coverage
#[tokio::test]
async fn test_set_level_all_levels() {
    let server = test_server();
    use rmcp::model::LoggingLevel;

    for level in [
        LoggingLevel::Debug,
        LoggingLevel::Info,
        LoggingLevel::Notice,
        LoggingLevel::Warning,
        LoggingLevel::Error,
        LoggingLevel::Critical,
        LoggingLevel::Alert,
        LoggingLevel::Emergency,
    ] {
        // Would need mock RequestContext - challenging due to rmcp internals
    }
}
```

---

## High Priority Improvements

### 1. Add clippy.toml Configuration

**Current**: No clippy.toml - relying on Cargo.toml lints only.

**Recommendation**: Add `clippy.toml` for fine-grained control:

```toml
# clippy.toml
cognitive-complexity-threshold = 15
disallowed-methods = [
    { path = "std::env::set_var", reason = "Use Config struct" },
    { path = "std::process::exit", reason = "Return Result instead" },
]
```

**Location**: Project root

**Benefit**: Prevents accidental use of unsafe patterns, enforces complexity limits.

---

### 2. Standardize Error Types with thiserror

**Current**: Mixed error handling - some functions return `String`, others use `ErrorData`.

**Files affected**:
- `src/server.rs:244-250` - Tool errors return `Result<String, String>`
- `src/prompts/templates.rs` - Uses `McpError::invalid_request`

**Recommendation**: Create a unified error type:

```rust
// src/error.rs
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ServerError {
    #[error("Tool execution failed: {0}")]
    ToolError(String),

    #[error("Resource not found: {uri}")]
    ResourceNotFound { uri: String },

    #[error("Invalid argument: {0}")]
    InvalidArgument(String),

    #[error("Division by zero")]
    DivisionByZero,
}

impl From<ServerError> for rmcp::ErrorData {
    fn from(err: ServerError) -> Self {
        rmcp::ErrorData::invalid_request(err.to_string(), None)
    }
}
```

**Benefit**: Type-safe errors, better error messages, consistent handling.

---

### 3. Add Examples Directory

**Current**: No examples - users must read tests or lib.rs docs.

**Recommendation**: Add practical examples:

```
examples/
├── basic_client.rs      # Minimal client connection
├── with_auth.rs         # Using API key authentication
└── testcontainers.rs    # Integration with testcontainers-rs
```

**Example `examples/basic_client.rs`**:

```rust
//! Basic MCP client connection example.
//!
//! Run with: cargo run --example basic_client

use reqwest::Client;
use serde_json::json;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = Client::new();

    // Initialize connection
    let response = client
        .post("http://localhost:3000/mcp")
        .header("Content-Type", "application/json")
        .json(&json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": {"name": "example", "version": "1.0"}
            }
        }))
        .send()
        .await?;

    println!("Server response: {}", response.text().await?);
    Ok(())
}
```

**Benefit**: Lower barrier to entry, demonstrates usage patterns.

---

## Medium Priority Improvements

### 4. Add Benchmarks

**Current**: No benchmarks directory.

**Recommendation**: Add performance benchmarks with criterion:

```toml
# Cargo.toml additions
[dev-dependencies]
criterion = "0.5"

[[bench]]
name = "tools"
harness = false
```

```rust
// benches/tools.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use mcp_test_server::{Config, McpTestServer};

fn bench_hash_sha256(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let server = McpTestServer::new(Config::default());

    c.bench_function("hash_sha256", |b| {
        b.iter(|| {
            rt.block_on(async {
                // Benchmark hash function
            })
        })
    });
}

criterion_group!(benches, bench_hash_sha256);
criterion_main!(benches);
```

**Benefit**: Track performance regressions, identify optimization opportunities.

---

### 5. Type Aliases for Complex Types

**Current**: Some complex nested types in server.rs.

**File**: `src/server.rs:83-84`

```rust
// Current
log_level: std::sync::Arc<std::sync::atomic::AtomicU8>,
```

**Recommendation**: Use type aliases for clarity:

```rust
// At module level
type LogLevel = Arc<AtomicU8>;

// In struct
log_level: LogLevel,
```

**Benefit**: Improved readability, easier refactoring.

---

### 6. Builder Pattern for Config

**Current**: Direct struct construction with defaults.

**Recommendation**: Add builder for complex configurations:

```rust
impl Config {
    pub fn builder() -> ConfigBuilder {
        ConfigBuilder::default()
    }
}

#[derive(Default)]
pub struct ConfigBuilder {
    host: Option<IpAddr>,
    port: Option<u16>,
    api_key: Option<String>,
    log_level: Option<String>,
}

impl ConfigBuilder {
    pub fn host(mut self, host: IpAddr) -> Self {
        self.host = Some(host);
        self
    }

    pub fn port(mut self, port: u16) -> Self {
        self.port = Some(port);
        self
    }

    pub fn api_key(mut self, key: impl Into<String>) -> Self {
        self.api_key = Some(key.into());
        self
    }

    pub fn build(self) -> Config {
        Config {
            host: self.host.unwrap_or_else(|| "0.0.0.0".parse().unwrap()),
            port: self.port.unwrap_or(3000),
            api_key: self.api_key,
            log_level: self.log_level.unwrap_or_else(|| "info".to_string()),
        }
    }
}
```

**Benefit**: Fluent API, better ergonomics for tests.

---

## Low Priority / Nice-to-Have

### 7. Structured Logging Enhancement

**Current**: Using tracing with basic fields.

**Recommendation**: Add request ID tracking:

```rust
// In auth_middleware
use uuid::Uuid;

let request_id = Uuid::new_v4();
let span = tracing::info_span!("request", id = %request_id);
let _guard = span.enter();
```

**Benefit**: Better observability, request correlation.

---

### 8. Integration Test Helpers Module

**Current**: Tests use inline setup code.

**Recommendation**: Create `tests/common/mod.rs` test utilities:

```rust
// tests/common/mod.rs
use mcp_test_server::{Config, McpTestServer};
use tokio::net::TcpListener;

pub struct TestServer {
    pub url: String,
    pub config: Config,
    shutdown: tokio::sync::oneshot::Sender<()>,
}

impl TestServer {
    pub async fn start() -> Self {
        let config = Config {
            port: 0, // Random available port
            ..Default::default()
        };
        // Setup and return
        todo!()
    }

    pub async fn with_auth(api_key: &str) -> Self {
        todo!()
    }
}

impl Drop for TestServer {
    fn drop(&mut self) {
        // Send shutdown signal
    }
}
```

**Benefit**: DRY test setup, consistent test environments.

---

### 9. Feature Flags for Optional Components

**Current**: All features always compiled.

**Recommendation**: Add feature flags for transport types:

```toml
[features]
default = ["sse", "streamable-http"]
sse = []
streamable-http = []
full = ["sse", "streamable-http"]
```

**Benefit**: Smaller binaries for specific use cases, faster compilation.

---

## What's Already Good

The codebase already follows many best practices:

1. **Strong typing** - `unsafe_code = "forbid"` in lints
2. **Comprehensive clippy lints** - `all`, `pedantic`, `nursery` warnings
3. **Good test coverage** - 85%+ threshold enforced
4. **Security practices** - Constant-time comparison, origin validation
5. **Release optimizations** - LTO, single codegen unit, strip symbols
6. **Documentation** - Module docs, function docs with `# Errors` sections
7. **`#[must_use]` annotations** - On relevant functions
8. **`const fn` where applicable** - `requires_auth()`, `unsubscribe()`

---

## Implementation Priority

| Item | Effort | Impact | Priority |
|------|--------|--------|----------|
| clippy.toml | Low | Medium | P1 |
| Error types | Medium | High | P1 |
| Examples | Medium | High | P1 |
| Benchmarks | Medium | Medium | P2 |
| Type aliases | Low | Low | P3 |
| Config builder | Medium | Low | P3 |
| Structured logging | Low | Low | P3 |
| Test helpers | Medium | Medium | P2 |
| Feature flags | High | Low | P4 |

---

## Next Steps

1. Create GitHub issues for P1 items
2. Implement clippy.toml (quick win)
3. Design error type hierarchy
4. Write basic_client example
5. Add benchmark infrastructure
