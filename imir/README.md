# imir

**I**nfra **M**etrics **I**nsight **R**enderer - Generate rendering instructions for [lowlighter/metrics](https://github.com/lowlighter/metrics) targets.

[![License: MIT OR Apache-2.0](https://img.shields.io/badge/License-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE)

## Overview

`imir` is a Rust library and CLI tool for managing metrics dashboard configurations. It normalizes YAML target definitions, discovers repositories using IMIR badges, and synchronizes discovered repositories with your configuration.

## Features

- **Target Normalization**: Parse and validate YAML configuration into normalized JSON
- **Repository Discovery**: Find repositories using IMIR badges via GitHub Code Search
- **Automatic Sync**: Merge discovered repositories into existing configurations
- **Badge Generation**: Create deterministic SVG badges with JSON manifests
- **Robust API Calls**: Exponential backoff retry logic for GitHub API failures
- **Progress Tracking**: Real-time progress indicators for long-running operations
- **Structured Logging**: Configurable log levels (debug, info, warn, error)

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
imir = "0.1"
```

Or install the CLI tool:

```bash
cargo install imir
```

## CLI Usage

### Normalize Targets

Convert YAML configuration to normalized JSON:

```bash
imir targets --config targets/targets.yaml --pretty
```

### Discover Repositories

Find repositories using IMIR badges:

```bash
# Discover from all sources (badges + stargazers)
imir discover --token $GITHUB_TOKEN --source all --format json

# Discover only from badge users
imir discover --token $GITHUB_TOKEN --source badge --max-pages 5

# Customize search patterns
imir discover --token $GITHUB_TOKEN \
  --badge-pattern "myorg/metrics" \
  --metrics-pattern "/dashboards/" \
  --max-pages 10
```

### Sync Discovered Repositories

Automatically add discovered repositories to your configuration:

```bash
# Sync from all sources
imir sync --config targets/targets.yaml --token $GITHUB_TOKEN --source all

# Sync only from stargazers with custom config
imir sync --config targets/targets.yaml \
  --token $GITHUB_TOKEN \
  --source stargazers \
  --max-pages 3
```

### Generate Badge Assets

Create SVG badges and JSON manifests for targets:

```bash
imir badge generate \
  --config targets/targets.yaml \
  --target my-profile \
  --output metrics/
```

## Library Usage

### Normalize Configuration

```rust
use imir::{load_targets, TargetsDocument};

fn main() -> Result<(), imir::Error> {
    let document = load_targets("targets/targets.yaml")?;

    for target in &document.targets {
        println!("Target: {} ({})", target.slug, target.display_name);
        println!("  Path: {}", target.target_path);
        println!("  Kind: {:?}", target.kind);
    }

    Ok(())
}
```

### Discover Repositories

```rust
use imir::{DiscoveryConfig, discover_badge_users};

#[tokio::main]
async fn main() -> Result<(), masterror::AppError> {
    let token = std::env::var("GITHUB_TOKEN").unwrap();
    let config = DiscoveryConfig::default();

    let repos = discover_badge_users(&token, &config).await?;

    for repo in repos {
        println!("Found: {}/{}", repo.owner, repo.repository);
    }

    Ok(())
}
```

### Sync Targets

```rust
use imir::{DiscoveredRepository, sync_targets};
use std::path::Path;

fn main() -> Result<(), masterror::AppError> {
    let discovered = vec![
        DiscoveredRepository {
            owner: "octocat".to_string(),
            repository: "hello-world".to_string(),
        },
    ];

    let added = sync_targets(
        Path::new("targets/targets.yaml"),
        &discovered,
    )?;

    println!("Added {} new repositories", added);
    Ok(())
}
```

## Configuration

### Discovery Configuration

Customize repository discovery behavior:

```rust
use imir::{DiscoveryConfig, retry::RetryConfig};

let config = DiscoveryConfig {
    max_pages: 5,
    badge_url_pattern: "myorg/metrics-renderer".to_string(),
    metrics_path_pattern: "/custom/".to_string(),
    retry_config: RetryConfig {
        max_attempts: 5,
        initial_delay_ms: 500,
        backoff_factor: 1.5,
    },
};
```

### Target Configuration

Example `targets.yaml`:

```yaml
targets:
  - owner: octocat
    repository: metrics
    type: open_source
    slug: octocat-metrics
    display_name: Octocat's Metrics
    badge:
      style: classic
      widget:
        columns: 2
        alignment: center
        border_radius: 6
```

Supported target types:
- `profile`: User profile dashboard
- `open_source`: Public repository metrics
- `private_project`: Private repository metrics

## Environment Variables

- `GITHUB_TOKEN`: GitHub personal access token for API authentication
- `RUST_LOG`: Log level (e.g., `info`, `debug`, `imir=debug`)

## Error Handling

All operations return `Result` types with detailed error information:

```rust
use imir::{Error, load_targets};

match load_targets("targets.yaml") {
    Ok(document) => println!("Loaded {} targets", document.targets.len()),
    Err(Error::Io { path, .. }) => eprintln!("Failed to read file: {}", path),
    Err(Error::Parse { message, .. }) => eprintln!("YAML parse error: {}", message),
    Err(e) => eprintln!("Error: {}", e),
}
```

## Logging

Configure logging with `RUST_LOG`:

```bash
# Show all info-level logs
RUST_LOG=info imir discover --token $GITHUB_TOKEN

# Debug-level logs for imir only
RUST_LOG=imir=debug imir sync --config targets.yaml --token $GITHUB_TOKEN

# Trace everything
RUST_LOG=trace imir targets --config targets.yaml
```

## Performance

- **HashSet deduplication**: O(1) lookups instead of O(n²) iterations
- **Pre-allocated vectors**: Reduced heap allocations with `Vec::with_capacity()`
- **Exponential backoff**: Resilient to transient API failures
- **Progress indicators**: Real-time feedback for long operations

## Testing

Run the test suite:

```bash
cargo test

# With coverage
cargo test --all-features

# With logging
RUST_LOG=debug cargo test
```

## Contributing

Contributions are welcome! Please ensure:
- All tests pass: `cargo test`
- Zero clippy warnings: `cargo clippy -- -D warnings`
- Code is formatted: `cargo +nightly fmt`
- SPDX license headers on new files

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Related Projects

- [lowlighter/metrics](https://github.com/lowlighter/metrics) - GitHub metrics visualization tool
- [RAprogramm/infra-metrics-insight-renderer](https://github.com/RAprogramm/infra-metrics-insight-renderer) - Infrastructure for automated metrics rendering
