# IMIR

This repository hosts reusable GitHub Actions workflows for rendering [lowlighter/metrics](https://github.com/lowlighter/metrics) dashboards used across RAprogramm projects.

## Repository metrics workflow

Use `.github/workflows/render-repository.yml` to refresh repository dashboards based on the "repository" template. Supply the repository handle and optional overrides for the owner, target branch name, artifact filename, or destination path.

```yaml
jobs:
  example:
    uses: RAprogramm/infra-metrics-insight-renderer/.github/workflows/render-repository.yml@main
    with:
      target_repo: my-repository
      # branch_name: ci/metrics-refresh-my-repository
      # target_owner: RAprogramm
      # target_path: metrics/my-repository.svg
      # temp_artifact: .metrics-tmp/my-repository.svg
    secrets:
      CLASSIC: ${{ secrets.METRICS_TOKEN }}
```

The workflow automatically renders the repository metrics card, commits the refreshed SVG to the configured path, and opens an idempotent pull request when changes are detected.

### Open-source repositories bundle

Workflows targeting public repositories that live under the `RAprogramm` organization can reuse `.github/workflows/render-open-source.yml`. The workflow accepts a JSON array with repository names and renders the standard repository dashboard for each entry.

```yaml
jobs:
  open_source:
    uses: RAprogramm/infra-metrics-insight-renderer/.github/workflows/render-open-source.yml@main
    with:
      repositories: '["masterror", "telegram-webapp-sdk"]'
    secrets:
      CLASSIC: ${{ secrets.METRICS_TOKEN }}
```

Providing a custom list of repositories allows a single job to refresh multiple metrics cards without duplicating boilerplate workflow definitions.
