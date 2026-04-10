<!--
SPDX-FileCopyrightText: 2025 RAprogramm <andrey.rozanov.vl@gmail.com>

SPDX-License-Identifier: MIT
-->

# Performance Tuning Guide

This guide covers performance optimization techniques and best practices for `imir`.

## Overview

`imir` is designed for blazing fast performance with minimal resource usage:

- **Zero-cost abstractions** - Rust compile-time guarantees
- **Minimal allocations** - Pre-allocated buffers and string operations
- **Parallel processing** - Multi-core badge generation with rayon
- **Efficient deduplication** - O(1) HashSet lookups
- **Retry with backoff** - Resilient API calls without hammering endpoints

## Benchmarks

Current performance metrics on standard hardware (Intel i7, 16GB RAM):

| Operation | Time | Throughput |
|-----------|------|------------|
| Parse small target | ~20μs | 50,000 ops/sec |
| Parse 100 targets | ~480μs | 2,083 configs/sec |
| Parse complex target | ~19μs | 52,631 ops/sec |
| Generate badge (single) | ~2ms | 500 badges/sec |
| Generate badges (parallel) | ~10ms for 10 | N-core speedup |

## Optimization Strategies

### 1. Parallel Badge Generation

**Problem**: Generating badges sequentially is slow for many targets.

**Solution**: Use parallel generation to utilize all CPU cores.

```bash
# Sequential (slow for many targets)
for slug in $(imir targets --config targets.yaml | jq -r '.[].slug'); do
    imir badge generate --target "$slug"
done

# Parallel (fast, utilizes all cores)
imir badge generate-all --config targets.yaml --output metrics/
```

**Performance gain**: N-core speedup where N = number of CPU cores.

### 2. Discovery Configuration

**Problem**: Discovering repositories scans too many pages.

**Solution**: Tune `max-pages` based on your needs.

```bash
# Fast: Limited scope
imir discover --token $GITHUB_TOKEN --max-pages 3

# Balanced: Default
imir discover --token $GITHUB_TOKEN --max-pages 10

# Thorough: Comprehensive
imir discover --token $GITHUB_TOKEN --max-pages 50
```

**Trade-off**: More pages = more repositories found, but slower execution.

### 3. API Retry Strategy

**Problem**: Transient failures slow down operations.

**Solution**: Configure retry parameters for your network conditions.

```rust
use imir::retry::RetryConfig;

// Fast but less resilient
let fast_config = RetryConfig {
    max_attempts: 2,
    initial_delay_ms: 500,
    backoff_factor: 1.5,
};

// Balanced (default)
let balanced_config = RetryConfig::default();  // 3 attempts, 1s, 2.0x

// Resilient but slower
let resilient_config = RetryConfig {
    max_attempts: 5,
    initial_delay_ms: 2000,
    backoff_factor: 2.5,
};
```

### 4. Memory Optimization

**Problem**: High memory usage during discovery.

**Solution**: Process in batches or reduce scope.

```bash
# Reduce concurrent allocations
imir discover --token $GITHUB_TOKEN --source badge --max-pages 5

# Process stargazers separately if needed
imir discover --token $GITHUB_TOKEN --source stargazers --max-pages 3
```

**Memory usage**: ~1MB per 100 discovered repositories.

### 5. I/O Optimization

**Problem**: Slow file I/O for badge generation.

**Solution**: Use SSD storage and batch writes.

```bash
# Use fast storage for output
imir badge generate-all --output /mnt/ssd/metrics/

# Ensure output directory exists (avoids extra syscalls)
mkdir -p metrics/
imir badge generate-all --output metrics/
```

## Profiling

### CPU Profiling with Flamegraph

Identify hot code paths:

```bash
# Install flamegraph
cargo install flamegraph

# Profile badge generation
cargo flamegraph --bin imir -- badge generate-all --config targets.yaml

# Open flamegraph.svg in browser
```

### Memory Profiling with Valgrind

Track memory allocations:

```bash
# Install valgrind (Linux)
sudo apt-get install valgrind

# Profile memory usage
valgrind --tool=massif \
  cargo run --release -- discover --token $GITHUB_TOKEN

# Analyze results
ms_print massif.out.*
```

### Benchmarking with Criterion

Run performance regression tests:

```bash
# Run all benchmarks
cargo bench

# Run specific benchmark
cargo bench parse_targets

# Compare with baseline
cargo bench --save-baseline main
git checkout feature-branch
cargo bench --baseline main
```

Expected output:
```
parse_targets_small     time:   [19.420 µs 20.036 µs 20.743 µs]
parse_100_targets       time:   [464.38 µs 482.09 µs 502.62 µs]
```

## Build Optimizations

### Release Profile

Ensure maximum optimization for production:

```toml
[profile.release]
lto = true              # Link-time optimization
codegen-units = 1       # Maximum optimization, slower builds
opt-level = 3           # Aggressive optimizations
```

Build for release:

```bash
cargo build --release

# Binary location
./target/release/imir
```

### Platform-Specific Optimizations

Target specific CPU features:

```bash
# Native CPU features
RUSTFLAGS="-C target-cpu=native" cargo build --release

# Specific features (e.g., AVX2)
RUSTFLAGS="-C target-feature=+avx2" cargo build --release
```

**Warning**: Binary won't be portable to older CPUs.

## API Rate Limiting

### GitHub API Limits

- **Unauthenticated**: 60 requests/hour
- **Authenticated**: 5,000 requests/hour
- **Search API**: 30 requests/minute (authenticated)

### Strategies to Avoid Limits

1. **Always use authentication**:
   ```bash
   imir discover --token $GITHUB_TOKEN
   ```

2. **Reduce API calls**:
   ```bash
   # Fewer pages = fewer API calls
   imir discover --token $GITHUB_TOKEN --max-pages 3
   ```

3. **Cache results**:
   ```bash
   # Save discovery results
   imir discover --token $GITHUB_TOKEN > discovered.json

   # Reuse without re-querying API
   cat discovered.json | jq '.[].repository'
   ```

4. **Respect rate limits**:
   - Built-in retry logic handles rate limit errors
   - Exponential backoff prevents hammering the API
   - Progress indicators show operation status

## Resource Usage

### Typical Resource Requirements

| Operation | CPU | Memory | Disk I/O |
|-----------|-----|--------|----------|
| Parse targets | Low | <10 MB | Minimal |
| Discovery (100 repos) | Low | ~100 MB | None |
| Badge generation (10) | Medium | <50 MB | Low |
| Parallel badges (100) | High | <200 MB | Medium |

### Monitoring Resource Usage

```bash
# CPU and memory monitoring
time RUST_LOG=info imir badge generate-all --config targets.yaml

# Detailed system monitoring (Linux)
/usr/bin/time -v imir discover --token $GITHUB_TOKEN

# Output includes:
# - Maximum resident set size (memory)
# - User/system CPU time
# - I/O statistics
```

## Performance Checklist

Before deployment, verify:

- [ ] **Release build**: Using `--release` flag
- [ ] **LTO enabled**: Check `Cargo.toml` profile
- [ ] **Benchmarks passing**: `cargo bench` shows no regression
- [ ] **Parallel execution**: Using `generate-all` for multiple badges
- [ ] **API token set**: `GITHUB_TOKEN` environment variable
- [ ] **Retry configured**: Appropriate for your network
- [ ] **Storage ready**: SSD recommended for badge output
- [ ] **Memory adequate**: 512MB+ for large operations

## Advanced Optimizations

### Custom Allocator

For production, consider using jemalloc:

```toml
[dependencies]
jemallocator = "0.5"
```

```rust
#[global_allocator]
static GLOBAL: jemallocator::Jemalloc = jemallocator::Jemalloc;
```

### Lazy Static Initialization

Reuse compiled regexes and allocations:

```rust
use once_cell::sync::Lazy;
use regex::Regex;

static SLUG_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"[^a-z0-9-]+").unwrap()
});
```

### Pre-allocated Buffers

Reduce allocations in hot loops:

```rust
// Instead of:
let mut results = Vec::new();
for item in items {
    results.push(process(item));
}

// Use:
let mut results = Vec::with_capacity(items.len());
for item in items {
    results.push(process(item));
}
```

## Troubleshooting Performance

### Slow Discovery

**Symptom**: Discovery takes minutes instead of seconds.

**Diagnosis**:
```bash
RUST_LOG=debug imir discover --token $GITHUB_TOKEN --max-pages 1
```

**Solutions**:
- Reduce `max-pages`
- Check network latency
- Verify API rate limits not exceeded
- Use `--source badge` instead of `all`

### High Memory Usage

**Symptom**: Process uses >1GB RAM.

**Diagnosis**:
```bash
/usr/bin/time -v imir discover --token $GITHUB_TOKEN
# Check "Maximum resident set size"
```

**Solutions**:
- Reduce `max-pages`
- Process in batches
- Check for memory leaks (shouldn't happen in safe Rust)

### Slow Badge Generation

**Symptom**: Badge generation takes >10s per badge.

**Diagnosis**:
```bash
time imir badge generate --target test-slug
```

**Solutions**:
- Use release build: `cargo build --release`
- Use parallel generation: `generate-all`
- Check disk I/O (use SSD)
- Profile with flamegraph

## Performance Monitoring

### Continuous Benchmarking

Set up automated benchmarks:

```yaml
# .github/workflows/benchmarks.yml
name: Benchmarks

on:
  push:
    branches: [main]
  pull_request:

jobs:
  benchmark:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Run benchmarks
        run: cargo bench
      - name: Store results
        uses: benchmark-action/github-action-benchmark@v1
        with:
          tool: 'cargo'
          output-file-path: target/criterion/*/new/estimates.json
```

### Metrics Collection

Track performance over time:

```bash
# Baseline
cargo bench --save-baseline main

# After changes
cargo bench --baseline main

# Results show percentage change
# Example: "change: [-5.2% -2.1% +1.3%]" (regression if positive)
```

## Conclusion

Performance is a feature, not an afterthought. Follow these guidelines to ensure `imir` runs blazing fast in production:

1. **Use release builds** with LTO
2. **Enable parallel processing** where applicable
3. **Configure retry logic** for your environment
4. **Monitor and profile** regularly
5. **Benchmark** before and after changes

For additional help, see [TROUBLESHOOTING.md](TROUBLESHOOTING.md).
