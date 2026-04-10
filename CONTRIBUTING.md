<!--
SPDX-FileCopyrightText: 2025 RAprogramm <andrey.rozanov.vl@gmail.com>

SPDX-License-Identifier: MIT
-->

# Contributing to IMIR

Thank you for your interest in contributing to IMIR!

## Development Setup

### Prerequisites

- Rust 1.90+ (edition 2024)
- `cargo-nextest` for testing
- `cargo-llvm-cov` for coverage
- `reuse` tool for license compliance
- `cargo-audit` for security audits

### Installation

```bash
# Install required tools
cargo install cargo-nextest cargo-llvm-cov cargo-audit
pip install reuse

# Clone repository
git clone https://github.com/RAprogramm/infra-metrics-insight-renderer.git
cd infra-metrics-insight-renderer/imir

# Run tests
cargo nextest run --all-features
```

## Development Workflow

### 1. Before Starting

- Check existing issues and PRs to avoid duplication
- For major changes, open an issue first to discuss
- Follow the [AI Development Protocol v2.1](https://github.com/RAprogramm/infra-metrics-insight-renderer)

### 2. Making Changes

```bash
# Create branch from issue number
git checkout -b 42

# Make changes following code standards
cargo +nightly fmt
cargo clippy --all-targets --all-features
cargo nextest run --all-features

# Commit with conventional commits
git commit -m "#42 feat: add new feature"
```

### 3. Code Standards

#### Formatting
- Use `cargo +nightly fmt` with provided `.rustfmt.toml`
- Line limit: 99 characters
- No comments except doc comments (`///`)

#### Documentation
- All public APIs must have doc comments
- Include examples in doc comments
- Explain arguments, return values, and errors

#### Testing
- Minimum 95% code coverage (target 100%)
- Unit tests for all functions
- Integration tests for public APIs
- Property-based tests with `proptest` where applicable
- Test error cases and edge cases

#### Error Handling
- Use `masterror` for all errors
- No `unwrap()` or `expect()` in production code
- Return `Result<T, Error>` for fallible operations

### 4. Commit Messages

Follow conventional commits format:

```
#<issue> <type>: <description>

<optional body>
```

**Types:**
- `feat`: New feature
- `fix`: Bug fix
- `refactor`: Code refactoring
- `test`: Adding tests
- `docs`: Documentation changes
- `chore`: Maintenance tasks

**Example:**
```
#42 feat: add badge generation for private repositories

Implements SVG badge generation with custom gradients for private
repository targets. Includes comprehensive tests and documentation.
```

### 5. Pull Request Process

1. Ensure all CI checks pass locally:
   ```bash
   cargo +nightly fmt --check
   cargo clippy --all-targets --all-features
   cargo nextest run --all-features
   cargo doc --no-deps --all-features
   cargo audit
   reuse lint
   ```

2. Push branch and create PR:
   - Title: Same as branch number
   - Description: Include "Closes #<issue>"
   - Reference related issues
   - Describe changes and reasoning

3. Address review feedback:
   - Respond to all comments
   - Make requested changes
   - Update tests if needed

4. After merge:
   - Branch is automatically deleted
   - Close related issues if not auto-closed

## CI Pipeline

All PRs must pass:

- **Quick Checks** (fmt, audit, license)
- **Validation Matrix** (clippy, tests, docs, build)
- **Coverage** (minimum 95%)
- **Benchmarks** (no performance regression)

## Project Structure

```
imir/
├── src/
│   ├── lib.rs          # Public API surface
│   ├── config.rs       # YAML configuration
│   ├── normalizer.rs   # Target normalization
│   ├── discover.rs     # Repository discovery
│   ├── badge.rs        # SVG badge generation
│   └── ...
├── tests/              # Integration tests
├── benches/            # Benchmarks
└── Cargo.toml
```

## Getting Help

- Check [README.md](README.md) for usage documentation
- Review existing issues and discussions
- Ask questions in issue comments
- Email: andrey.rozanov.vl@gmail.com

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
