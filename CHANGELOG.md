# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [1.1.0] - 2026-01-30

### Bug Fixes

- Filter release-plz output for GitHub Actions
- Use release-plz CLI with registry-manifest-path for unpublished packages
- Add packages write permission for Docker release
- Simplify release-plz workflow to use action directly ([#16](https://github.com/nazq/test_mcp_servers/pull/16))
- Use release-plz CLI with registry-manifest-path for unpublished packages
- Use git tag for release-plz comparison on unpublished packages
- Add features_always_increment_minor for 0.x releases
- Update release-plz config to match working pattern
- Revert release-plz to standard pattern
- Quote if expression for YAML parsing
- Only cut release when release PR is merged
- Resolve CI clippy warnings and add dependabot

### CI/CD

- Add workflow_dispatch trigger to release-plz

### Dependencies

- Consolidate dependency updates

### Features

- Add MCP Apps protocol support with UI resource tools
- Release v1.0.0 - SSE transport removed ([#24](https://github.com/nazq/test_mcp_servers/pull/24)) **BREAKING**
- Clarify Docker image tag in documentation ([#17](https://github.com/nazq/test_mcp_servers/pull/17))
- Add Docker image reference to crate documentation ([#14](https://github.com/nazq/test_mcp_servers/pull/14))
- Add noop tool for testing tool invocation
- Add MCP test server implementation

### Performance

- Use Swatinem/rust-cache and amd64-only Docker builds


## [0.4.0] - 2025-12-08

### Features

- Release v1.0.0 - SSE transport removed ([#24](https://github.com/nazq/test_mcp_servers/pull/24)) **BREAKING**


## [0.3.1] - 2025-12-08

### Performance

- Use Swatinem/rust-cache and amd64-only Docker builds


## [0.3.0] - 2025-12-08

### Bug Fixes

- Filter release-plz output for GitHub Actions
- Use release-plz CLI with registry-manifest-path for unpublished packages
- Add packages write permission for Docker release
- Simplify release-plz workflow to use action directly ([#16](https://github.com/nazq/test_mcp_servers/pull/16))

### Features

- Clarify Docker image tag in documentation ([#17](https://github.com/nazq/test_mcp_servers/pull/17))


## [0.2.0] - 2025-12-08

### Bug Fixes

- Use release-plz CLI with registry-manifest-path for unpublished packages
- Use git tag for release-plz comparison on unpublished packages

### Features

- Add Docker image reference to crate documentation ([#14](https://github.com/nazq/test_mcp_servers/pull/14))

## [0.1.0] - 2025-12-07

### 🚀 Features

- Add MCP test server implementation

### 🐛 Bug Fixes

- Resolve CI clippy warnings and add dependabot

### 💼 Other

- Consolidate dependency updates

### ⚙️ Miscellaneous Tasks

- Add Cargo.lock for reproducible builds

### Added

- Initial project structure
- MCP test server with SSE and Streamable HTTP transports
- 25 test tools for comprehensive client testing
- 8 test resources including static and dynamic content
- 5 test prompts with argument support
- API key authentication via Bearer token
- Docker image for testcontainers integration
