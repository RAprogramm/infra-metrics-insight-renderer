<!--
SPDX-FileCopyrightText: 2025 RAprogramm <andrey.rozanov.vl@gmail.com>

SPDX-License-Identifier: MIT
-->

<h1 align="center">IMIR</h1>
<div align="right">
  <blockquote><em>Infra Metrics Insight Renderer</em></blockquote>
</div>

<hr />

<p align="center">
  <a href="https://hitsofcode.com/github/RAprogramm/infra-metrics-insight-renderer/view?branch=main">
    <img src="https://hitsofcode.com/github/RAprogramm/infra-metrics-insight-renderer?branch=main" alt="Hits-of-Code" />
  </a>
</p>

<p align="center">
  <a href="./assets/imir.png">
    <img src="./assets/imir.png" alt="IMIR" />
  </a>
</p>

## What is IMIR?

IMIR is a Rust CLI tool for automated GitHub metrics generation and repository discovery. It provides commands for:

- **GitHub CLI operations**: Create PRs, manage issues via `gh` integration
- **Git automation**: Commit, push, branch management
- **Metrics generation**: Render repository and profile dashboards using [lowlighter/metrics](https://github.com/lowlighter/metrics)
- **Discovery**: Automatically detect repositories using IMIR badges
- **Badge generation**: Create lightweight SVG badges for discovered targets
- **README updates**: Maintain user tables with `imir readme` command

## Quick Start

### For Users

Add IMIR badge to your repository README:

```markdown
<!-- For public repositories -->
[![IMIR](https://raw.githubusercontent.com/RAprogramm/infra-metrics-insight-renderer/main/assets/badges/imir-badge-simple-public.svg)](https://github.com/RAprogramm/infra-metrics-insight-renderer)

<!-- For private repositories -->
[![IMIR](https://raw.githubusercontent.com/RAprogramm/infra-metrics-insight-renderer/main/assets/badges/imir-badge-simple-private.svg)](https://github.com/RAprogramm/infra-metrics-insight-renderer)

<!-- For GitHub profiles -->
[![IMIR](https://raw.githubusercontent.com/RAprogramm/infra-metrics-insight-renderer/main/assets/badges/imir-badge-simple-profile.svg)](https://github.com/RAprogramm/infra-metrics-insight-renderer)
```

Add metrics placeholder:

```markdown
![Metrics](https://raw.githubusercontent.com/RAprogramm/infra-metrics-insight-renderer/main/metrics/<repo-name>.svg)
```

Star this repository and wait for automatic discovery (runs daily at 02:00 UTC).

See [WORKFLOW.md](WORKFLOW.md) for detailed discovery and metrics generation flow.

## Registered Users

IMIR automatically discovers and tracks users who add badges to their repositories.

<details>
<summary>Profile badges</summary>

<!-- IMIR will update this table automatically -->

<table>
  <thead>
    <tr><th>Account</th><th>Badge</th></tr>
  </thead>
  <tbody>
    <tr>
      <td><code>RAprogramm</code></td>
      <td><img alt="RAprogramm profile metrics" src="https://raw.githubusercontent.com/RAprogramm/infra-metrics-insight-renderer/main/metrics/profile.svg" /></td>
    </tr>
  </tbody>
</table>

</details>

<details>
<summary>Open-source repositories</summary>

<!-- IMIR will update this table automatically -->

<table>
  <thead>
    <tr><th>Repository</th><th>Badge</th></tr>
  </thead>
  <tbody>
    <tr>
      <td><code>RAprogramm/masterror</code></td>
      <td><img alt="masterror metrics" src="https://raw.githubusercontent.com/RAprogramm/infra-metrics-insight-renderer/main/metrics/masterror.svg" /></td>
    </tr>
    <tr>
      <td><code>RAprogramm/telegram-webapp-sdk</code></td>
      <td><img alt="telegram-webapp-sdk metrics" src="https://raw.githubusercontent.com/RAprogramm/infra-metrics-insight-renderer/main/metrics/telegram-webapp-sdk.svg" /></td>
    </tr>
    <tr>
      <td><code>RAprogramm/infra-metrics-insight-renderer</code></td>
      <td><img alt="infra-metrics-insight-renderer metrics" src="https://raw.githubusercontent.com/RAprogramm/infra-metrics-insight-renderer/main/metrics/infra-metrics-insight-renderer.svg" /></td>
    </tr>
  </tbody>
</table>

</details>

<details>
<summary>Private repositories</summary>

<!-- IMIR will update this table automatically -->

<p>
  Private dashboards follow the same embedding rules. Publish badges from this section once private projects are registered.
</p>

</details>

## CLI Usage

### Configuration Management

```bash
# Load and normalize targets from YAML
imir --config targets/targets.yaml --pretty

# Validate open-source repository inputs
imir open-source --input '[{"repository": "masterror"}]'
```

### Discovery Operations

```bash
# Discover repositories using IMIR badges
imir discover --token $GITHUB_TOKEN --source all --format json

# Sync discovered repositories to targets.yaml
imir sync --config targets/targets.yaml --token $GITHUB_TOKEN --source all
```

### Badge Generation

```bash
# Generate badge assets for specific target
imir badge generate --config targets/targets.yaml --target profile --output metrics

# Generate all badge assets in parallel
imir badge generate-all --config targets/targets.yaml --output metrics
```

### README Updates

```bash
# Update README.md with current user tables
imir readme --readme README.md --config targets/targets.yaml
```

### Git and GitHub Operations

```bash
# Commit and push changes
imir git commit-push --branch ci/update --path metrics/profile.svg --message "Update metrics"

# Create or update pull request
imir gh pr-create --repo owner/repo --head feature --base main --title "Title" --body "Body" --labels ci --token $GITHUB_TOKEN
```

### Render Input Normalization

```bash
# Normalize profile render inputs
imir render normalize-profile --target-user RAprogramm --display-name "Profile"

# Normalize repository render inputs
imir render normalize-repository --target-repo masterror --target-owner RAprogramm --github-repo owner/repo
```

See [imir/README.md](imir/README.md) for complete CLI documentation.

## Development

### Local Validation

```bash
# Run full CI validation pipeline
./scripts/ci-check.sh
```

Includes:
- Formatting (`cargo +nightly fmt`)
- Linting (`cargo clippy --all-targets --all-features -- -D warnings`)
- Tests (`cargo test --all-features`)
- Documentation (`cargo doc --no-deps --all-features`)
- Security audit (`cargo audit`)
- Dependency checks (`cargo deny check`)
- License compliance (`reuse lint`)

### Project Structure

```
metrics-renderer/
├── imir/                    # Rust CLI crate
│   ├── src/
│   │   ├── main.rs         # CLI entry point
│   │   ├── config.rs       # YAML configuration parsing
│   │   ├── normalizer.rs   # Target normalization logic
│   │   ├── discover.rs     # Repository discovery
│   │   ├── badge.rs        # Badge SVG generation
│   │   ├── readme.rs       # README table updates
│   │   ├── gh.rs           # GitHub CLI operations
│   │   └── git.rs          # Git operations
│   └── Cargo.toml
├── targets/
│   └── targets.yaml        # Metrics targets configuration
├── .github/workflows/       # Automated rendering workflows
├── assets/badges/          # Static badge SVG files
├── metrics/                # Generated metrics dashboards
├── WORKFLOW.md             # Discovery flow documentation
└── scripts/ci-check.sh     # Local CI validation script
```

## Storage Strategy

**Current**: SVG artifacts committed directly to `metrics/` directory. GitHub Actions render updated metrics on schedule, commit to main branch, and serve via `raw.githubusercontent.com`.

**Trade-offs**:
- ✅ Zero infrastructure, immediate availability
- ⚠️ Git history noise (mitigated with `chore(metrics):` prefix)
- ⚠️ Repository size growth (negligible for typical refresh rates)

**Future**: Migrate to database backend (PostgreSQL/SQLite) for historical queries and reduced git noise. Migration path straightforward since rendering logic isolated in CLI.

## Release Process

1. Run `./scripts/ci-check.sh` locally
2. Create annotated tag: `git tag -a v0.1.0`
3. Push tag: `git push origin v0.1.0`
4. Draft GitHub release (triggers `.github/workflows/release.yml`)
5. Workflow builds and uploads `imir-x86_64-unknown-linux-gnu.tar.gz` binary
6. Update downstream workflows to download new release

## License

MIT OR Apache-2.0
