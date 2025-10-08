# Configuration Guide

This document describes all configuration options available in `imir`.

## Table of Contents

- [Target Configuration](#target-configuration)
- [Discovery Configuration](#discovery-configuration)
- [Badge Configuration](#badge-configuration)
- [Environment Variables](#environment-variables)

## Target Configuration

### File Format

Targets are defined in YAML format (`targets.yaml`):

```yaml
targets:
  - owner: octocat
    repository: metrics
    type: open_source
    slug: octocat-metrics
    display_name: "Octocat's Metrics Dashboard"
    branch_name: main
    target_path: metrics/octocat-metrics.svg
    badge:
      style: classic
      widget:
        columns: 2
        alignment: center
        border_radius: 6
```

### Required Fields

| Field | Type | Description |
|-------|------|-------------|
| `owner` | string | GitHub username or organization name |
| `type` | enum | Target type: `profile`, `open_source`, or `private_project` |

### Optional Fields

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `repository` | string | null | Repository name (required for `open_source` and `private_project`) |
| `slug` | string | auto-generated | Unique identifier for the target |
| `display_name` | string | derived from slug | Human-readable name |
| `branch_name` | string | `main` | Branch to commit metrics to |
| `contributors_branch` | string | `main` | Branch for contributor analysis |
| `target_path` | string | auto-generated | Path to output SVG file |
| `temp_artifact` | string | auto-generated | Temporary file path |
| `time_zone` | string | `UTC` | Timezone for metrics |
| `include_private` | boolean | `false` | Include private repositories (profile only) |

### Target Types

#### Profile

Metrics for a GitHub user profile.

```yaml
- owner: octocat
  type: profile
  include_private: false
```

Special rules:
- `repository` must not be set
- `include_private` defaults to `true` only for owner `RAprogramm`

#### Open Source

Metrics for a public repository.

```yaml
- owner: octocat
  repository: hello-world
  type: open_source
```

Requirements:
- `repository` is required

#### Private Project

Metrics for a private repository.

```yaml
- owner: acme-corp
  repository: internal-api
  type: private_project
```

Requirements:
- `repository` is required
- Requires appropriate GitHub token permissions

## Discovery Configuration

### CLI Options

```bash
imir discover \
  --token $GITHUB_TOKEN \
  --source all \
  --max-pages 10 \
  --badge-pattern "myorg/metrics" \
  --metrics-pattern "/custom/"
```

### Parameters

| Flag | Type | Default | Description |
|------|------|---------|-------------|
| `--token` | string | `$GITHUB_TOKEN` | GitHub personal access token |
| `--source` | enum | `all` | Discovery source: `badge`, `stargazers`, or `all` |
| `--format` | enum | `json` | Output format: `json` or `yaml` |
| `--max-pages` | number | `10` | Maximum pages to fetch from GitHub API |
| `--badge-pattern` | string | `RAprogramm/infra-metrics-insight-renderer` | Badge URL pattern to search for |
| `--metrics-pattern` | string | `/metrics/` | Metrics path pattern to search for |

### Programmatic Configuration

```rust
use imir::{DiscoveryConfig, retry::RetryConfig};

let config = DiscoveryConfig {
    max_pages: 5,
    badge_url_pattern: "myorg/metrics-renderer".to_string(),
    metrics_path_pattern: "/dashboards/".to_string(),
    retry_config: RetryConfig {
        max_attempts: 5,
        initial_delay_ms: 500,
        backoff_factor: 1.5,
    },
};
```

### Retry Configuration

Control exponential backoff behavior for API calls:

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `max_attempts` | u32 | `3` | Maximum retry attempts before failing |
| `initial_delay_ms` | u64 | `1000` | Initial delay between retries (milliseconds) |
| `backoff_factor` | f64 | `2.0` | Multiplier for exponential backoff |

Example retry sequence with defaults:
1. First attempt (immediate)
2. Wait 1000ms, retry
3. Wait 2000ms, retry
4. Wait 4000ms, fail if unsuccessful

## Badge Configuration

### Badge Styles

Available styles:
- `classic`: Traditional GitHub badge style (default)
- `flat`: Flat design without gradients
- `flat-square`: Flat design with square edges

```yaml
badge:
  style: flat-square
```

### Widget Configuration

Control badge widget appearance:

```yaml
badge:
  widget:
    columns: 2        # Number of columns (1-4)
    alignment: center # Alignment: left, center, right
    border_radius: 6  # Border radius in pixels (0-16)
```

| Field | Type | Range | Default | Description |
|-------|------|-------|---------|-------------|
| `columns` | number | 1-4 | `2` | Number of widget columns |
| `alignment` | enum | `left`, `center`, `right` | `center` | Widget alignment |
| `border_radius` | number | 0-16 | `6` | Border radius (pixels) |

## Environment Variables

### GITHUB_TOKEN

GitHub personal access token for API authentication.

**Required for:**
- Repository discovery
- Sync operations
- Private repository access

**Scopes needed:**
- `public_repo`: Access public repositories
- `repo`: Access private repositories (if using `private_project` targets)

**Usage:**
```bash
export GITHUB_TOKEN=ghp_xxxxxxxxxxxx
imir discover --source all
```

### RUST_LOG

Configure logging verbosity.

**Levels:**
- `error`: Only error messages
- `warn`: Warnings and errors
- `info`: Informational messages (default)
- `debug`: Detailed debugging information
- `trace`: Very detailed trace information

**Examples:**
```bash
# Show all info-level logs
RUST_LOG=info imir discover --token $GITHUB_TOKEN

# Debug imir only, info for dependencies
RUST_LOG=imir=debug,info imir sync --config targets.yaml --token $GITHUB_TOKEN

# Trace everything
RUST_LOG=trace imir targets --config targets.yaml
```

**Module-specific logging:**
```bash
# Debug only discovery module
RUST_LOG=imir::discover=debug imir discover --token $GITHUB_TOKEN

# Trace retry logic
RUST_LOG=imir::retry=trace imir sync --config targets.yaml --token $GITHUB_TOKEN
```

## Examples

### Minimal Configuration

Simplest target definition:

```yaml
targets:
  - owner: octocat
    type: profile
```

Auto-generated values:
- `slug`: `octocat`
- `display_name`: `octocat`
- `branch_name`: `main`
- `target_path`: `metrics/octocat.svg`

### Full Configuration

All options specified:

```yaml
targets:
  - owner: acme-corp
    repository: api-server
    type: open_source
    slug: acme-api
    display_name: "ACME API Server Metrics"
    branch_name: metrics
    contributors_branch: develop
    target_path: dashboards/api-server.svg
    temp_artifact: temp/api-server-temp.svg
    time_zone: America/New_York
    badge:
      style: flat-square
      widget:
        columns: 3
        alignment: left
        border_radius: 12
```

### Multiple Targets

Mix different target types:

```yaml
targets:
  # User profile
  - owner: octocat
    type: profile
    include_private: true

  # Public repository
  - owner: acme-corp
    repository: public-sdk
    type: open_source
    display_name: "ACME Public SDK"

  # Private project
  - owner: acme-corp
    repository: internal-tools
    type: private_project
    display_name: "Internal Tools"
```

## Validation Rules

### Slug Uniqueness

All slugs must be unique within a configuration file:

```yaml
# ❌ Invalid: duplicate slugs
targets:
  - owner: user1
    type: profile
    slug: metrics  # duplicate!

  - owner: user2
    type: profile
    slug: metrics  # duplicate!
```

### Branch Name Uniqueness

All branch names must be unique:

```yaml
# ❌ Invalid: duplicate branch names
targets:
  - owner: user1
    type: profile
    branch_name: main  # duplicate!

  - owner: user2
    type: profile
    branch_name: main  # duplicate!
```

### Path Uniqueness

Target paths and temp artifacts must be unique:

```yaml
# ❌ Invalid: duplicate paths
targets:
  - owner: user1
    type: profile
    target_path: metrics/dashboard.svg  # duplicate!

  - owner: user2
    type: profile
    target_path: metrics/dashboard.svg  # duplicate!
```

## Best Practices

1. **Use explicit slugs** for production configurations to avoid auto-generation changes
2. **Set display_name** for better readability in dashboards
3. **Keep max_pages reasonable** (5-10) to avoid rate limiting
4. **Use module-specific logging** in production for better performance
5. **Rotate GitHub tokens** regularly for security
6. **Version control** your `targets.yaml` file
7. **Test configurations** with `imir targets --config targets.yaml` before deploying

## Troubleshooting

### Rate Limiting

If you hit GitHub API rate limits:
- Reduce `--max-pages` value
- Increase retry delays: `retry_config.initial_delay_ms`
- Use authenticated requests (always provide `--token`)

### Validation Errors

Common validation issues:
- **Missing repository**: `open_source` and `private_project` require `repository` field
- **Duplicate slugs**: Use unique `slug` values for each target
- **Invalid ranges**: Check `columns` (1-4) and `border_radius` (0-16)

### Discovery Issues

If discovery returns no results:
- Verify badge pattern matches your setup
- Check metrics path pattern
- Increase `--max-pages` value
- Verify GitHub token has correct permissions
