# CI/CD Pipeline Documentation

This document describes the CI/CD pipeline for the MCP Test Server project.

## Overview

The project uses three GitHub Actions workflows:

1. **CI Pipeline** (`ci.yml`) - Runs on pull requests and pushes to main
2. **Release Pipeline** (`release.yml`) - Publishes releases when tags are pushed
3. **Release Please** (`release-plz.yml`) - Automates version bumps and changelog generation

## CI Pipeline

**Trigger**: Pull requests to main, pushes to main

### Jobs

1. **Format Check** (`fmt`)
   - Verifies code formatting with `cargo fmt`
   - Fast feedback on formatting issues

2. **Clippy Lints** (`clippy`)
   - Runs on Rust 1.85.0 (MSRV) and stable
   - All warnings treated as errors
   - Checks all targets and features

3. **Test Suite** (`test`)
   - Runs on Rust 1.85.0 (MSRV) and stable
   - Executes all tests with all features enabled
   - Uses cargo caching for faster builds

4. **Code Coverage** (`coverage`)
   - Generates coverage report using `cargo-llvm-cov`
   - Enforces 85% minimum coverage threshold
   - Uploads coverage to Codecov (requires `CODECOV_TOKEN` secret)

5. **Docker Build Test** (`docker`)
   - Builds Docker image without pushing
   - Validates Dockerfile and build process
   - Uses GitHub Actions cache for layers

6. **Status Check** (`status`)
   - Aggregates results from all jobs
   - Ensures all checks passed

### Caching Strategy

The CI pipeline caches:
- Cargo registry (`~/.cargo/registry`)
- Cargo git index (`~/.cargo/git`)
- Build artifacts (`target/`)

Cache keys include Cargo.lock hash for automatic invalidation on dependency changes.

## Release Pipeline

**Trigger**: Tag pushes matching `v*` (e.g., `v1.0.0`)

### Jobs

1. **Publish Docker Images** (`publish-docker`)
   - Builds multi-architecture images (amd64, arm64)
   - Pushes to `ghcr.io/nazq/mcp-test-server`
   - Tags:
     - Semantic version (`v1.2.3`)
     - Major.minor (`v1.2`)
     - Major (`v1`)
     - `latest` (for default branch)
   - Uses GitHub Container Registry (no additional secrets needed)

2. **Create GitHub Release** (`create-release`)
   - Extracts changelog from CHANGELOG.md
   - Creates GitHub release with release notes
   - Runs after Docker publishing succeeds

## Release Please Automation

**Trigger**: Pushes to main branch

### Functionality

The `release-plz` workflow:
1. Analyzes commits since last release
2. Determines version bump based on conventional commits
3. Updates CHANGELOG.md
4. Creates a release PR or releases directly
5. Creates git tags when ready

### Conventional Commits

Commit messages should follow this format:

```
<type>(<scope>): <description>

[optional body]

[optional footer]
```

**Types**:
- `feat`: New feature (minor version bump)
- `fix`: Bug fix (patch version bump)
- `perf`: Performance improvement
- `refactor`: Code refactoring
- `docs`: Documentation changes
- `test`: Test changes
- `build`: Build system changes
- `ci`: CI configuration changes
- `chore`: Miscellaneous changes
- `deps`: Dependency updates

**Breaking Changes**: Add `BREAKING CHANGE:` in footer (major version bump)

### Configuration

- `release-plz.toml`: Main configuration
- `changelog.toml`: Changelog generation rules

## Required Secrets

Add these secrets in GitHub repository settings:

### Optional
- `CODECOV_TOKEN`: Codecov upload token
  - Get from https://codecov.io
  - Only needed if using Codecov for coverage reports

### Automatically Provided
- `GITHUB_TOKEN`: Automatically provided by GitHub Actions
  - Used for release-plz, Docker registry, and GitHub releases
  - No configuration needed

## Docker Images

### Registry
Images are published to GitHub Container Registry:
```
ghcr.io/nazq/mcp-test-server
```

### Tags
- `v1.2.3`: Specific version
- `v1.2`: Minor version
- `v1`: Major version
- `latest`: Latest stable release

### Pulling Images
```bash
# Pull specific version
docker pull ghcr.io/nazq/mcp-test-server:v1.0.0

# Pull latest
docker pull ghcr.io/nazq/mcp-test-server:latest
```

### Multi-Architecture Support
Images are built for:
- `linux/amd64` (x86_64)
- `linux/arm64` (ARM64/Apple Silicon)

## Workflow Permissions

### CI Workflow
- `contents: read` (default)

### Release Workflow
- `contents: write` (for creating releases)
- `packages: write` (for pushing Docker images)

### Release Please Workflow
- `contents: write` (for updating CHANGELOG.md and creating tags)
- `pull-requests: write` (for creating release PRs)

## Local Testing

### Run CI Checks Locally
```bash
# Format check
cargo fmt --all -- --check

# Clippy
cargo clippy --all-targets --all-features -- -D warnings

# Tests
cargo test --all-features --verbose

# Coverage
cargo install cargo-llvm-cov
cargo llvm-cov --all-features --workspace --lcov --output-path lcov.info

# Docker build
docker build -t mcp-test-server:test .
```

### Test Release Process
```bash
# Install release-plz
cargo install release-plz

# Dry run
release-plz release-pr --dry-run
```

## Troubleshooting

### Coverage Fails
- Ensure tests have adequate coverage
- Minimum threshold is 85%
- Add more tests or adjust threshold in `ci.yml`

### Docker Build Fails
- Check Dockerfile syntax
- Ensure all dependencies are in Cargo.toml
- Verify multi-stage build works locally

### Release Fails
- Ensure version in Cargo.toml matches tag
- Check GITHUB_TOKEN has correct permissions

### release-plz Not Creating PRs
- Verify conventional commit format
- Check GITHUB_TOKEN permissions
- Ensure commits exist since last release

## Best Practices

1. **Commit Messages**: Always use conventional commits format
2. **Testing**: Maintain >85% code coverage
3. **Formatting**: Run `cargo fmt` before committing
4. **Linting**: Fix all clippy warnings
5. **Dependencies**: Keep dependencies up to date
6. **Versioning**: Let release-plz handle version bumps
7. **Changelog**: Don't manually edit CHANGELOG.md

## References

- [Conventional Commits](https://www.conventionalcommits.org/)
- [release-plz Documentation](https://release-plz.dev/)
- [GitHub Actions Documentation](https://docs.github.com/en/actions)
- [cargo-llvm-cov](https://github.com/taiki-e/cargo-llvm-cov)
