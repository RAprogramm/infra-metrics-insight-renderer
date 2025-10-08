# Troubleshooting Guide

This guide covers common issues and their solutions when using `imir`.

## Table of Contents

- [Installation Issues](#installation-issues)
- [GitHub API Issues](#github-api-issues)
- [Configuration Errors](#configuration-errors)
- [Discovery Problems](#discovery-problems)
- [Badge Generation Issues](#badge-generation-issues)
- [Performance Issues](#performance-issues)

## Installation Issues

### Rust Version Too Old

**Problem**: Compilation fails with edition-related errors.

```
error: edition "2024" is unstable and only available with -Z unstable-options
```

**Solution**: Update to the latest Rust stable version:

```bash
rustup update stable
rustc --version  # Should be 1.80+
```

### Missing Dependencies

**Problem**: Linker errors during build.

**Solution**: Install required system dependencies:

**Ubuntu/Debian:**
```bash
sudo apt-get install pkg-config libssl-dev
```

**Fedora/RHEL:**
```bash
sudo dnf install pkgconfig openssl-devel
```

**macOS:**
```bash
brew install openssl pkg-config
```

## GitHub API Issues

### Rate Limiting

**Problem**: Discovery or sync operations fail with rate limit errors.

```
Error: GitHub API rate limit exceeded
```

**Solution**:

1. **Use authenticated requests** (automatic with `GITHUB_TOKEN`):
   - Unauthenticated: 60 requests/hour
   - Authenticated: 5000 requests/hour

2. **Reduce `max-pages`** parameter:
   ```bash
   imir discover --token $GITHUB_TOKEN --max-pages 3
   ```

3. **Wait for rate limit reset**:
   ```bash
   # Check when rate limit resets
   curl -H "Authorization: token $GITHUB_TOKEN" \
     https://api.github.com/rate_limit
   ```

4. **Use retry logic** (built-in):
   - Default: 3 attempts with exponential backoff
   - Automatically retries on transient failures

### Authentication Failures

**Problem**: `401 Unauthorized` or `403 Forbidden` errors.

```
Error: failed to initialize GitHub client: authentication failed
```

**Solution**:

1. **Verify token exists**:
   ```bash
   echo $GITHUB_TOKEN
   ```

2. **Check token permissions** (needs `repo` scope):
   - Go to https://github.com/settings/tokens
   - Ensure token has `public_repo` or `repo` scope
   - Regenerate if necessary

3. **Pass token explicitly**:
   ```bash
   imir discover --token "ghp_yourtoken..."
   ```

4. **Check token expiration**:
   - GitHub PATs can expire
   - Create new token if expired

### Network Timeouts

**Problem**: Operations hang or timeout.

```
Error: request timeout after 30s
```

**Solution**:

1. **Check network connectivity**:
   ```bash
   curl -I https://api.github.com
   ```

2. **Use retry with longer delays**:
   ```rust
   use imir::retry::RetryConfig;

   let config = RetryConfig {
       max_attempts: 5,
       initial_delay_ms: 2000,  // 2 seconds
       backoff_factor: 2.0,
   };
   ```

3. **Reduce concurrent operations**:
   - Use `--max-pages 1` for testing
   - Process repositories sequentially

## Configuration Errors

### Invalid YAML Syntax

**Problem**: Configuration parsing fails.

```
Error: failed to parse configuration: invalid YAML at line 5
```

**Solution**:

1. **Validate YAML syntax**:
   ```bash
   # Use yamllint or online validators
   yamllint targets/targets.yaml
   ```

2. **Common YAML mistakes**:
   ```yaml
   # WRONG: Tab indentation
   targets:
   	- owner: octocat

   # CORRECT: Space indentation
   targets:
     - owner: octocat
   ```

   ```yaml
   # WRONG: Missing quotes for special characters
   display_name: Project: Dashboard

   # CORRECT: Quoted strings
   display_name: "Project: Dashboard"
   ```

### Missing Required Fields

**Problem**: Validation errors for required fields.

```
Error: invalid configuration: repository names cannot be empty strings
```

**Solution**: Ensure all required fields are present:

```yaml
targets:
  - owner: octocat           # Required
    repository: hello-world  # Required for open_source/private_project
    type: open_source        # Required (profile, open_source, private_project)
```

### Duplicate Targets

**Problem**: Multiple targets with same owner/repository.

```
Error: duplicate target: octocat/hello-world
```

**Solution**: Remove duplicate entries or use unique slugs:

```yaml
targets:
  - owner: octocat
    repository: hello-world
    type: open_source
    slug: octocat-hello-world-main

  - owner: octocat
    repository: hello-world
    type: open_source
    branch_name: develop
    slug: octocat-hello-world-dev  # Different slug
```

## Discovery Problems

### No Repositories Found

**Problem**: Discovery returns empty list.

```
Badge discovery complete: 0 repositories found
```

**Solution**:

1. **Verify badge pattern**:
   ```bash
   # Check actual badge URLs in use
   imir discover --token $GITHUB_TOKEN \
     --badge-pattern "RAprogramm/infra-metrics-insight-renderer" \
     --metrics-pattern "/metrics/"
   ```

2. **Increase search scope**:
   ```bash
   # Try all sources
   imir discover --token $GITHUB_TOKEN --source all --max-pages 10
   ```

3. **Search manually** to verify badges exist:
   ```bash
   # GitHub Code Search
   curl -H "Authorization: token $GITHUB_TOKEN" \
     "https://api.github.com/search/code?q=RAprogramm+infra-metrics-insight-renderer"
   ```

### Discovery Too Slow

**Problem**: Discovery takes too long.

**Solution**:

1. **Reduce max-pages**:
   ```bash
   imir discover --token $GITHUB_TOKEN --max-pages 3
   ```

2. **Use specific source**:
   ```bash
   # Badge search is faster than stargazers
   imir discover --token $GITHUB_TOKEN --source badge
   ```

3. **Monitor progress**:
   ```bash
   RUST_LOG=info imir discover --token $GITHUB_TOKEN
   ```

## Badge Generation Issues

### Missing Output Directory

**Problem**: Badge generation fails with I/O error.

```
Error: failed to write badge artifact: No such file or directory
```

**Solution**: Create output directory first:

```bash
mkdir -p metrics
imir badge generate --config targets.yaml --target my-slug --output metrics/
```

### Invalid Target Slug

**Problem**: Target not found error.

```
Error: target 'my-slug' was not found
```

**Solution**:

1. **List available targets**:
   ```bash
   imir targets --config targets.yaml --pretty | jq '.[].slug'
   ```

2. **Use correct slug**:
   ```bash
   imir badge generate --config targets.yaml --target octocat-metrics
   ```

3. **Check slug normalization**:
   - Slugs are lowercase alphanumeric with hyphens
   - `"My Project"` becomes `"my-project"`

### Permission Denied

**Problem**: Cannot write badge files.

```
Error: failed to write badge artifact: Permission denied
```

**Solution**:

1. **Check directory permissions**:
   ```bash
   ls -ld metrics/
   chmod 755 metrics/
   ```

2. **Use different output directory**:
   ```bash
   imir badge generate --output /tmp/metrics/
   ```

## Performance Issues

### High Memory Usage

**Problem**: Process consumes excessive memory.

**Solution**:

1. **Process targets in batches**:
   ```bash
   # Instead of generate-all, use individual commands
   for slug in $(imir targets --config targets.yaml | jq -r '.[].slug'); do
       imir badge generate --target "$slug"
   done
   ```

2. **Reduce concurrent operations**:
   ```bash
   # Use sequential processing instead of parallel
   imir badge generate  # Instead of generate-all
   ```

3. **Monitor resource usage**:
   ```bash
   RUST_LOG=debug imir discover --token $GITHUB_TOKEN 2>&1 | \
     grep -E "memory|allocation"
   ```

### Slow Badge Generation

**Problem**: Badge generation is slow.

**Solution**:

1. **Use parallel generation**:
   ```bash
   imir badge generate-all --config targets.yaml --output metrics/
   ```

2. **Check release build**:
   ```bash
   cargo build --release
   ./target/release/imir badge generate-all
   ```

3. **Profile performance**:
   ```bash
   cargo bench
   ```

### Benchmark Regression

**Problem**: Performance degrades after changes.

**Solution**:

1. **Run benchmarks**:
   ```bash
   cargo bench
   ```

2. **Compare with baseline**:
   ```bash
   # Expected performance:
   # parse_targets_small:   ~20μs
   # parse_100_targets:     ~480μs
   # parse_complex_target:  ~19μs
   ```

3. **Profile with flamegraph**:
   ```bash
   cargo install flamegraph
   cargo flamegraph --bench benchmarks
   ```

## Logging and Debugging

### Enable Debug Logging

See detailed operation logs:

```bash
# All debug logs
RUST_LOG=debug imir discover --token $GITHUB_TOKEN

# Only imir logs
RUST_LOG=imir=debug imir sync --config targets.yaml --token $GITHUB_TOKEN

# Trace level for maximum detail
RUST_LOG=trace imir targets --config targets.yaml
```

### Test Configuration

Verify configuration without side effects:

```bash
# Parse and validate only
imir targets --config targets.yaml --pretty

# Dry-run discovery
imir discover --token $GITHUB_TOKEN --source badge --max-pages 1
```

## Getting Help

If issues persist:

1. **Check existing issues**: https://github.com/RAprogramm/infra-metrics-insight-renderer/issues
2. **Enable debug logging** and include output when reporting
3. **Provide minimal reproduction** case
4. **Include versions**:
   ```bash
   imir --version
   rustc --version
   ```

## Common Error Messages

| Error | Meaning | Solution |
|-------|---------|----------|
| `repository names cannot be empty` | Missing required field | Add `repository` field to target |
| `failed to fetch contributor stats` | GitHub API error | Check token permissions and rate limits |
| `contributors_branch cannot contain whitespace` | Invalid branch name | Use valid Git branch name (e.g., `main`, `develop`) |
| `target 'X' was not found` | Invalid slug | List targets with `imir targets --config` |
| `failed to parse targets config` | YAML syntax error | Validate YAML syntax |
| `GitHub code search failed` | API failure | Retry with exponential backoff (automatic) |
