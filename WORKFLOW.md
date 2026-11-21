<!--
SPDX-FileCopyrightText: 2025 RAprogramm <andrey.rozanov.vl@gmail.com>

SPDX-License-Identifier: MIT
-->

# IMIR Discovery and Metrics Generation Workflow

This document describes the target workflow and the current legacy implementation.

## Target Flow (GitHub App)

1. **Install App**: User installs the IMIR GitHub App and selects the account/repositories to include.
2. **Star**: User stars `RAprogramm/infra-metrics-insight-renderer`.
3. **Auto PR**: IMIR opens a pull request in the user's README with the IMIR badge and metrics block. No manual edits, no user token.
4. **Merge**: User merges the pull request.
5. **Render**: IMIR renders metrics (profile, open-source repository, or private repository) using a GitHub App installation token (`ghs_`) with read-only permissions.
6. **Store & serve**: SVG is committed to `metrics/<slug>.svg` in this repository and served via `raw.githubusercontent.com`.
7. **Refresh**: Metrics refresh once per day and on relevant push/webhook events.

## Current Flow (Legacy)

The current implementation still uses a classic PAT (`secrets.CLASSIC`) for rendering and discovery. Migration to the GitHub App flow is in progress.

### Discovery

- Runs daily at 02:00 UTC (`sync-targets.yml`).
- Scans stargazers of this repository.
- Checks the first page of each stargazer's repositories for IMIR badges in README files.
- Detected targets are added to `targets/targets.yaml`.

### Rendering

- Runs daily at 03:00 UTC (`render-all.yml`).
- Renders targets with `lowlighter/metrics`.
- Commits SVG artifacts to `metrics/`.
- Serves images via `raw.githubusercontent.com`.

## Configuration Files

### `targets/targets.yaml`

Contains all discovered and manually registered targets:

```yaml
targets:
  - owner: RAprogramm
    type: profile
    slug: profile
  - owner: RAprogramm
    repository: masterror
    type: open_source
    contributors_branch: main
```

### `README.md` User Tables

IMIR maintains discoverable user tables in README.md inside `<details>` tags:

- **Profile badges**: List of GitHub users with profile dashboards
- **Open-source repositories**: List of public repositories with metrics
- **Private repositories**: List of private repositories/organizations with metrics

Tables are automatically updated by `imir readme` command during CI runs.

## Discovery Configuration

See `imir/src/discover.rs` for discovery implementation details:

- `discover_stargazer_repositories()`: Fetches stargazers and analyzes their repos
- `discover_badge_users()`: Searches for IMIR badges in README files
- `extract_repo_from_readme()`: Parses README content to detect badge and metrics placeholder

## Metrics Rendering Workflows

See `.github/workflows/` for automated rendering workflows:

- `render-all.yml`: Daily scheduled refresh for all targets
- `render-repository.yml`: Reusable workflow for repository metrics
- `render-open-source.yml`: Bundled workflow for public repositories
- `badge-sync.yml`: Generates badge SVGs into `assets/badges/generated` for changed slugs

## Future Enhancements

1. **Query parameters in badges**: Allow users to customize metrics via URL parameters
2. **Real-time metrics**: Reduce refresh interval for active repositories
3. **Custom templates**: Support user-provided metric templates
4. **API endpoint**: Provide REST API for programmatic access to metrics data
5. **Database backend**: Migrate from git-based storage to PostgreSQL for historical queries
