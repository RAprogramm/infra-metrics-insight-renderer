<!--
SPDX-FileCopyrightText: 2025 RAprogramm <andrey.rozanov.vl@gmail.com>

SPDX-License-Identifier: MIT
-->

# IMIR Discovery and Metrics Generation Workflow

This document describes the automated discovery and metrics generation flow.

## User Onboarding Flow

### Step 1: User adds badge to their README

User adds appropriate IMIR badge to their repository README:

- **Public repositories**: `imir-badge-simple-public.svg`
- **Private repositories**: `imir-badge-simple-private.svg`
- **GitHub profiles**: `imir-badge-simple-profile.svg`

### Step 2: User adds metrics placeholder

User adds placeholder where generated metrics will be displayed:

```markdown
![Metrics](https://raw.githubusercontent.com/RAprogramm/infra-metrics-insight-renderer/main/metrics/<repo-name>.svg)
```

### Step 3: IMIR checks who needs metrics

IMIR discovery system runs on schedule (daily at 02:00 UTC) to detect new users.

### Step 4: IMIR discovers repositories

Discovery process:

1. **Stargazers check**: Fetch users who starred `infra-metrics-insight-renderer`
2. **README analysis**: For each stargazer, fetch and parse their repository README
3. **Badge detection**: Search for IMIR badge URLs in README content
4. **Badge type identification**: Determine metrics type based on badge color/filename

### Step 5: IMIR finds badge in user repository

When badge is detected:

- Extract repository owner and name
- Identify badge type (public/private/profile)
- Validate repository exists and is accessible

### Step 6: IMIR generates metrics

Based on badge type, IMIR generates appropriate metrics using `lowlighter/metrics`:

- **Public repositories**: Languages, traffic, contributors
- **Private repositories**: Private insights (requires token with private access)
- **Profiles**: GitHub profile dashboard

**Future enhancement**: Support query parameters in badge URL to customize displayed metrics.

### Step 7: IMIR stores and serves metrics

Generated metrics workflow:

1. Render SVG dashboard via `lowlighter/metrics` Docker action
2. Commit SVG to `metrics/<slug>.svg` in this repository
3. User's placeholder automatically displays metrics via `raw.githubusercontent.com` URL
4. No additional infrastructure required

### Step 8: IMIR adds to daily refresh schedule

After successful initial generation:

1. Add target to `targets/targets.yaml` configuration
2. Target included in daily scheduled refresh (04:00 UTC)
3. Metrics automatically updated once per day
4. User always sees fresh metrics without manual intervention

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
- `badge-sync.yml`: Synchronizes lightweight badge placeholders

## Future Enhancements

1. **Query parameters in badges**: Allow users to customize metrics via URL parameters
2. **Real-time metrics**: Reduce refresh interval for active repositories
3. **Custom templates**: Support user-provided metric templates
4. **API endpoint**: Provide REST API for programmatic access to metrics data
5. **Database backend**: Migrate from git-based storage to PostgreSQL for historical queries
