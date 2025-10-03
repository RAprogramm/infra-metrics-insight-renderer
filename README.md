# IMIR
> _Infra Metrics Insight Renderer_

---

[![Hits-of-Code](https://hitsofcode.com/github/RAprogramm/infra-metrics-insight-renderer?branch=main)](https://hitsofcode.com/github/RAprogramm/infra-metrics-insight-renderer/view?branch=main)

This repository hosts reusable GitHub Actions workflows for rendering [lowlighter/metrics](https://github.com/lowlighter/metrics)
dashboards used across RAprogramm projects.

## Repository metrics workflow

Use `.github/workflows/render-repository.yml` to refresh repository dashboards based on the "repository" template. Supply the
repository handle and optional overrides for the owner, target branch name, artifact filename, or destination path.

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

The workflow automatically renders the repository metrics card, commits the refreshed SVG to the configured path, and opens an
idempotent pull request when changes are detected.

### Open-source repositories bundle

Workflows targeting public repositories that live under the `RAprogramm` organization can reuse `.github/workflows/render-open-source.yml`.
The workflow accepts a JSON array with repository names and renders the standard repository dashboard for each entry.

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

## Unified target configuration

The [`targets/targets.yaml`](targets/targets.yaml) file defines every metrics target that should be refreshed on the regular
schedule. Each entry requires the GitHub account (`owner`), an optional `repository`, and the `type` of metrics card to render:

- `profile` – render a classic GitHub profile card.
- `open_source` – render the repository template for public projects.
- `private_project` – render the repository template for private projects.

When the scheduled [`render-all.yml`](.github/workflows/render-all.yml) workflow runs it executes the
`metrics-orchestrator` CLI to transform the YAML into a matrix consumed by the rendering jobs. New targets can be tested locally
with:

```bash
cargo run --manifest-path metrics-orchestrator/Cargo.toml -- --config targets/targets.yaml --pretty
```

The command outputs the normalized JSON document that the workflow uses. The same CLI is invoked during CI, so validation errors
must be resolved locally before a workflow run succeeds.

Optional per-target overrides include:

- `branch_name` (or the alias `branch`) – select the Git branch used for the metrics refresh pull request.
- `target_path` – change where the rendered SVG is stored.
- `temp_artifact` – adjust the temporary filename produced by the renderer before moving it into place.
- `time_zone` – customize the time zone passed to the renderer.
- `slug` – override the derived slug used for filenames and workflow dispatch names.

Unset overrides fall back to deterministic defaults chosen by the orchestrator, so adding a new target only requires the owner,
repository (when applicable), and target type.

## IMIR badge integration

Register a repository or profile by adding a new entry to [`targets/targets.yaml`](targets/targets.yaml). The orchestrator
normalizes the identifier into a slug that becomes the SVG filename and forms the basis for badge embeds. Scheduled renders run
on the cadence defined in the repository workflows, so dashboards refresh automatically once the target is listed.

After the initial registration lands in `main`, trigger the on-demand workflow named `render-<slug>.yml` to produce the first
badge artifact. This pre-populates the SVG before linking to it in documentation.

Embed the rendered badge in Markdown using the slugged artifact path:

```markdown
![IMIR](https://raw.githubusercontent.com/RAprogramm/infra-metrics-insight-renderer/main/metrics/<slug>.svg)
```

Replace `<slug>` with the normalized identifier emitted for the target (for example, `owner-repository` for repository cards or
`owner` for profile cards). Once the slug exists under `metrics/`, the badge can be referenced from any README or documentation
page.

### Badge catalogue

The published badges are grouped by color so their category is obvious at a glance. Reuse the badges directly from the
repository to avoid stale snapshots.

#### 🟩 Open-source badges

| Repository | Badge |
| --- | --- |
| `RAprogramm/masterror` | ![masterror metrics](https://raw.githubusercontent.com/RAprogramm/infra-metrics-insight-renderer/main/metrics/masterror.svg) |
| `RAprogramm/telegram-webapp-sdk` | ![telegram-webapp-sdk metrics](https://raw.githubusercontent.com/RAprogramm/infra-metrics-insight-renderer/main/metrics/telegram-webapp-sdk.svg) |

#### 🟦 Private project badges

Private dashboards follow the same embedding rules. Publish badges from this section once private projects are registered.

#### 🟪 Profile badges

| Account | Badge |
| --- | --- |
| `RAprogramm` | ![RAprogramm profile metrics](https://raw.githubusercontent.com/RAprogramm/infra-metrics-insight-renderer/main/metrics/profile.svg) |

#### Color reference

- 🟩 Green badges indicate open-source repositories.
- 🟦 Blue badges denote private repositories.
- 🟪 Purple badges represent GitHub profile dashboards.

## metrics-orchestrator CLI

The `metrics-orchestrator` crate lives in [`metrics-orchestrator`](metrics-orchestrator). It validates the target configuration,
applies deterministic defaults for filenames, branch names, and time zones, and serializes the normalized targets in JSON form.
Unit tests cover slug normalization, configuration validation, and duplicate detection to ensure predictable behaviour when new
targets are added.

## Local development workflow

Use [`scripts/ci-check.sh`](scripts/ci-check.sh) to run the full validation pipeline locally. The helper script formats the code
with the nightly toolchain, executes Clippy, builds all targets, runs tests, generates documentation, and invokes `cargo audit`
and `cargo deny` to ensure dependency health. Install [`cargo-audit`](https://crates.io/crates/cargo-audit) and
[`cargo-deny`](https://crates.io/crates/cargo-deny) beforehand to enable the security checks.
