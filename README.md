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
      # temp_artifact: .metrics-tmp/repository.svg
    secrets:
      CLASSIC: ${{ secrets.METRICS_TOKEN }}
```

The workflow automatically renders the repository metrics card, commits the refreshed SVG to the configured path, and opens an idempotent pull request when changes are detected.
