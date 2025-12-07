# CI/CD Quick Start Guide

Get your CI/CD pipeline running in 5 minutes.

## Prerequisites

- Repository: `nazq/test_mcp_servers` on GitHub
- Admin access to repository settings

## Setup Steps

### Step 1: Enable GitHub Actions (If Needed)

1. Go to repository Settings → Actions → General
2. Under "Actions permissions", select "Allow all actions and reusable workflows"
3. Under "Workflow permissions", ensure:
   - "Read and write permissions" is selected
   - "Allow GitHub Actions to create and approve pull requests" is checked
4. Click "Save"

### Step 3: Test the CI Pipeline

1. **Create a test branch**:
   ```bash
   git checkout -b test-cicd
   ```

2. **Make a small change**:
   ```bash
   # Add a comment somewhere or update README
   echo "# CI/CD Test" >> README.md
   git add README.md
   git commit -m "test: verify CI/CD pipeline"
   git push origin test-cicd
   ```

3. **Create PR**:
   ```
   Go to: https://github.com/nazq/test_mcp_servers/pulls
   Click: "New pull request"
   Select: test-cicd → main
   Click: "Create pull request"
   ```

4. **Watch the checks**:
   - All 5 jobs should appear (fmt, clippy, test, coverage, docker)
   - Wait for all to complete (2-5 minutes)
   - All should show green checkmarks ✓

### Step 3: Optional - Add Codecov

1. **Get Codecov Token**:
   ```
   Visit: https://codecov.io
   Sign in with GitHub
   Add repository: nazq/test_mcp_servers
   Copy the upload token
   ```

2. **Add to GitHub**:
   ```
   Go to: https://github.com/nazq/test_mcp_servers/settings/secrets/actions
   Click: "New repository secret"
   Name: CODECOV_TOKEN
   Value: [paste token from Codecov]
   Click: "Add secret"
   ```

### Step 4: Test Release Process (After First PR Merged)

1. **Merge a PR with conventional commit**:
   ```bash
   # On your branch
   git commit -m "feat: add new feature"
   # Push and merge PR
   ```

2. **release-plz creates release PR**:
   - Automatically runs on push to main
   - Creates PR with version bump and CHANGELOG update
   - Review the PR

3. **Merge release PR**:
   - release-plz will create the version tag
   - Tag triggers release workflow
   - Publishes to ghcr.io

## Verification Checklist

After setup, verify:

- [ ] GitHub Actions is enabled with write permissions
- [ ] Test PR shows all 5 checks passing
- [ ] (Optional) `CODECOV_TOKEN` configured for coverage reports
- [ ] Can create releases with conventional commits

## Common Issues

### CI jobs don't start
**Fix**: Enable GitHub Actions in repository settings

### Coverage upload fails
**Fix**: Add `CODECOV_TOKEN` secret (or ignore if you don't want Codecov)

### release-plz doesn't create PR
**Fix**:
- Use conventional commit format (`feat:`, `fix:`, etc.)
- Ensure GITHUB_TOKEN has correct permissions in settings

## What Happens on Each Action

### Opening a PR
1. Format check runs (fast)
2. Clippy runs on 2 Rust versions
3. Tests run on 2 Rust versions
4. Coverage check enforces 85% threshold
5. Docker build validates Dockerfile
6. Status job shows aggregate result

### Merging to Main
1. CI runs again
2. release-plz analyzes commits
3. If version-worthy commits exist:
   - Creates/updates release PR
   - Updates CHANGELOG.md
   - Bumps version in Cargo.toml

### Merging Release PR
1. release-plz creates git tag (e.g., `v1.0.0`)
2. Tag push triggers release workflow:
   - Builds multi-arch Docker images
   - Pushes to ghcr.io/nazq/test_mcp_servers
   - Creates GitHub release

## Testing Locally

Before pushing, run local checks:

```bash
# Quick verification
.github/scripts/verify-ci.sh

# Individual checks
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features --verbose
docker build -t mcp-test-server:test .
```

## Success!

If you've completed all steps:
- ✅ CI/CD pipeline is active
- ✅ PRs are automatically checked
- ✅ Releases are automated
- ✅ Docker images are published
- ✅ Conventional commits control versioning

## Next Steps

1. **Document your commit conventions** for your team
2. **Set up branch protection rules** requiring CI to pass
3. **Add project-specific checks** if needed
4. **Monitor first few releases** to ensure smooth operation

## Support

- Full docs: `.github/CICD.md`
- Secrets guide: `.github/SECRETS.md`
- Summary: `CICD_SUMMARY.md`

## Links

- Repository: https://github.com/nazq/test_mcp_servers
- Docker Images: https://github.com/nazq/test_mcp_servers/pkgs/container/test_mcp_servers
- Actions: https://github.com/nazq/test_mcp_servers/actions
