# GitHub Secrets Configuration

This document lists all secrets required for the CI/CD pipeline to function properly.

## Optional Secrets

### CODECOV_TOKEN
**Required for**: Uploading coverage reports to Codecov
**Used in**: `ci.yml` workflow (coverage job)
**How to get**:
1. Log in to https://codecov.io with your GitHub account
2. Add the repository if not already added
3. Go to repository Settings
4. Copy the repository upload token

**How to add**:
1. Go to repository Settings → Secrets and variables → Actions
2. Click "New repository secret"
3. Name: `CODECOV_TOKEN`
4. Value: Paste the token from Codecov
5. Click "Add secret"

**Note**: The CI will not fail if this secret is missing; coverage upload will simply be skipped.

---

## Automatically Provided Secrets

### GITHUB_TOKEN
**Required for**:
- Creating releases
- Pushing Docker images to GHCR
- Creating release PRs with release-plz

**Used in**: All workflows
**How to get**: Automatically provided by GitHub Actions
**No configuration needed**

---

## Verifying Secrets

To verify secrets are set correctly:

1. Go to repository Settings → Secrets and variables → Actions
2. You should see:
   - `CODECOV_TOKEN` (Optional)
3. `GITHUB_TOKEN` is not visible here (automatically provided)

---

## Troubleshooting

### "Error: Unauthorized" when pushing Docker images
- This should not happen as `GITHUB_TOKEN` is automatic
- Verify the workflow has `packages: write` permission
- Check repository settings allow GitHub Actions

### Coverage upload fails
- This is non-fatal if `CODECOV_TOKEN` is not set
- To fix: Add `CODECOV_TOKEN` secret
- Or remove the Codecov upload step from `ci.yml`

### release-plz cannot create PRs
- Verify `GITHUB_TOKEN` has correct permissions
- Check workflow has `contents: write` and `pull-requests: write`
- Ensure branch protection rules allow GitHub Actions

---

## Security Best Practices

1. **Never commit secrets**: Secrets should only be in GitHub repository settings
2. **Rotate tokens regularly**: Update tokens periodically for security
3. **Use minimum permissions**: Only grant necessary scopes
4. **Monitor usage**: Check GitHub Actions logs for unauthorized access

---

## Quick Setup Checklist

- [ ] (Optional) Set up Codecov and added `CODECOV_TOKEN`
- [ ] Verified workflows have correct permissions in workflow files
- [ ] Tested CI pipeline with a test PR

---

## Contact

If you need help with secrets configuration:
1. Check GitHub Actions logs for specific error messages
2. Review workflow YAML files for permission requirements
3. Consult the CICD.md documentation
