<!--
SPDX-FileCopyrightText: 2025 RAprogramm <andrey.rozanov.vl@gmail.com>

SPDX-License-Identifier: MIT
-->

<a id="top"></a>
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

<p>
  This repository hosts reusable GitHub Actions workflows for rendering
  <a href="https://github.com/lowlighter/metrics">lowlighter/metrics</a>
  dashboards used across RAprogramm projects.
</p>

<p>
  <strong>IMIR exists to take the friction out of README polish.</strong>
  Point your repository at the provided workflows, merge the generated
  <code>metrics/&lt;slug&gt;.svg</code> badge, and drop a one-line embed into
  your README: GitHub instantly shows a fully styled metrics card and IMIR keeps
  it refreshed automatically. No custom infrastructure, no hand-crafted SVGs,
  just copy the badge URL and ship.
</p>

<h2>Table of contents</h2>
<ul>
  <li><a href="#repository-metrics-workflow">Repository metrics workflow</a>
    <ul>
      <li><a href="#open-source-repositories-bundle">Open-source repositories bundle</a></li>
    </ul>
  </li>
  <li><a href="#unified-target-configuration">Unified target configuration</a></li>
  <li><a href="#imir-badge-integration">IMIR badge integration</a>
    <ul>
      <li><a href="#badge-catalogue">Badge catalogue</a>
        <ul>
          <li><a href="#open-source-badges">🟩 Open-source badges</a></li>
          <li><a href="#private-project-badges">🟦 Private project badges</a></li>
          <li><a href="#profile-badges">🟪 Profile badges</a></li>
          <li><a href="#color-reference">Color reference</a></li>
        </ul>
      </li>
    </ul>
  </li>
  <li><a href="#imir-cli">IMIR CLI</a></li>
  <li><a href="#storage-strategy">Storage strategy</a></li>
  <li><a href="#local-development-workflow">Local development workflow</a></li>
  <li><a href="#release-process">Release process</a></li>
</ul>

<p align="right"><em><a href="#top">Back to top</a></em></p>

<h2 align="center" id="repository-metrics-workflow">Repository metrics workflow</h2>

<p>
  Use <code>.github/workflows/render-repository.yml</code> to refresh repository dashboards based on the "repository" template. Supply the
  repository handle and optional overrides for the owner, target branch name, artifact filename, or destination path.

  Repository cards now highlight two extra sections powered by <code>plugin_languages</code> and <code>plugin_traffic</code>:
  <ul>
    <li><strong>Most used languages</strong> &mdash; a GitHub-colored bar that focuses on the <code>most-used</code> segment so language mixes differ across repositories at a glance.</li>
    <li><strong>Traffic insights</strong> &mdash; a condensed view of recent views and clones to expose repository momentum directly on the badge.</li>
  </ul>
</p>

<pre><code class="language-yaml">jobs:
  example:
    uses: RAprogramm/infra-metrics-insight-renderer/.github/workflows/render-repository.yml@main
    with:
      target_repo: my-repository
      # branch_name: ci/metrics-refresh-my-repository
      # contributors_branch: main
      # target_owner: RAprogramm
      # target_path: metrics/my-repository.svg
      # temp_artifact: .metrics-tmp/my-repository.svg
    secrets:
      CLASSIC: ${{ secrets.METRICS_TOKEN }}</code></pre>

<p>
  The workflow automatically renders the repository metrics card, commits the refreshed SVG to the configured path, and opens an
  idempotent pull request when changes are detected.
</p>

<p align="right"><em><a href="#top">Back to top</a></em></p>

<h3 align="center" id="open-source-repositories-bundle">Open-source repositories bundle</h3>

<p>
  Workflows targeting public repositories that live under the <code>RAprogramm</code> organization can reuse
  <code>.github/workflows/render-open-source.yml</code>. The workflow accepts a JSON array with repository names and renders the standard repository dashboard for each entry. The list is validated through the <code>imir open-source</code> subcommand, ensuring the matrix only includes non-empty repository names.
</p>

<pre><code class="language-yaml">jobs:
  open_source:
    uses: RAprogramm/infra-metrics-insight-renderer/.github/workflows/render-open-source.yml@main
    with:
      repositories: '[{"repository": "masterror"}, {"repository": "telegram-webapp-sdk"}]'
    secrets:
      CLASSIC: ${{ secrets.METRICS_TOKEN }}</code></pre>

<p>
  Providing a custom list of repositories allows a single job to refresh multiple metrics cards without duplicating boilerplate workflow definitions.
</p>

<p align="right"><em><a href="#top">Back to top</a></em></p>

<h2 align="center" id="unified-target-configuration">Unified target configuration</h2>

<p>
  The <a href="targets/targets.yaml"><code>targets/targets.yaml</code></a> file defines every metrics target that should be refreshed on the regular
  schedule. Each entry requires the GitHub account (<code>owner</code>), an optional <code>repository</code>, and the <code>type</code> of metrics card to render:
</p>

<h3 align="center" id="badge-sync-workflow">Badge synchronization workflow</h3>

<p>
  Lightweight badge placeholders stay in sync through
  <a href=".github/workflows/badge-sync.yml"><code>.github/workflows/badge-sync.yml</code></a>.
  The workflow regenerates affected badges whenever README updates reference a new slug, on-demand through the
  <code>workflow_dispatch</code> entrypoint, and nightly at 04:00 UTC via the scheduled trigger. Each run calls
  <code>imir badge generate --target &lt;slug&gt;</code> for every impacted entry and stores the resulting SVG and JSON manifest under
  <code>metrics/</code>.
</p>

<p>
  Scheduled and main-branch invocations push badge refreshes to the dedicated <code>ci/badge-sync</code> branch. The automation
  opens or updates a pull request labeled <code>ci</code> and <code>badges</code>, so the repository always exposes the latest placeholders
  without manual intervention. Pull request runs execute in validation mode without committing, ensuring contributors receive
  immediate feedback if README changes reference unknown slugs.
</p>

<p>
  The continuous integration pipeline now includes a smoke test that invokes <code>imir badge generate</code> against the
  <code>profile</code> target. The test fails fast if CLI changes break badge generation, preventing regressions from entering the
  workflow.
</p>

<ul>
  <li><code>profile</code> – render a classic GitHub profile card.</li>
  <li><code>open_source</code> – render the repository template for public projects.</li>
  <li><code>private_project</code> – render the repository template for private projects.</li>
</ul>

<p>
  When the scheduled <a href=".github/workflows/render-all.yml"><code>render-all.yml</code></a> workflow runs it executes the
  <code>imir</code> CLI to transform the YAML into a matrix consumed by the rendering jobs. New targets can be tested locally
  with:
</p>

<pre><code class="language-bash">cargo run --manifest-path imir/Cargo.toml -- --config targets/targets.yaml --pretty</code></pre>

<p>
  Use the open-source helper to normalize ad-hoc repository lists for the bundled workflow:
</p>

<pre><code class="language-bash">cargo run --manifest-path imir/Cargo.toml -- open-source --input '[{"repository": "masterror"}, {"repository": "telegram-webapp-sdk"}]'</code></pre>

<p>
  The command outputs repository descriptors containing the slugged name and the contributors branch analyzed by the renderer.
  The same CLI is invoked during CI, so validation errors must be resolved locally before a workflow run succeeds.
</p>

<p>Optional per-target overrides include:</p>
<ul>
  <li><code>branch_name</code> (or the alias <code>branch</code>) – select the Git branch used for the metrics refresh pull request.</li>
  <li><code>target_path</code> – change where the rendered SVG is stored.</li>
  <li><code>temp_artifact</code> – adjust the temporary filename produced by the renderer before moving it into place.</li>
  <li><code>contributors_branch</code> – specify the repository branch analyzed by the contributors plugin.</li>
  <li><code>time_zone</code> – customize the time zone passed to the renderer.</li>
  <li><code>slug</code> – override the derived slug used for filenames and workflow dispatch names.</li>
  <li><code>include_private</code> – set to <code>true</code> to include private repositories and secret achievements for the target. Profile cards owned by <code>RAprogramm</code> enable this flag by default so the dashboard reflects private activity without additional configuration.</li>
</ul>

<p>
  Contributor branch overrides are especially helpful for repositories that use a non-<code>main</code> default branch: the
  generated metrics now highlight contributors for the correct branch without manual workflow edits.
</p>

<p>
  Unset overrides fall back to deterministic defaults chosen by the orchestrator, so adding a new target only requires the owner,
  repository (when applicable), and target type.
</p>

<p align="right"><em><a href="#top">Back to top</a></em></p>

<h2 align="center" id="imir-badge-integration">IMIR badge integration</h2>

<h3 align="center">Quick start for users</h3>

<p>
  <strong>No configuration needed!</strong> Simply add the badge to your README and star ⭐ this repository:
</p>

<pre><code class="language-markdown">![Metrics](https://raw.githubusercontent.com/RAprogramm/infra-metrics-insight-renderer/main/metrics/YOUR-REPO-NAME.svg)</code></pre>

<p>
  Replace <code>YOUR-REPO-NAME</code> with your repository name (e.g., <code>masterror</code> for <code>RAprogramm/masterror</code>).
  The automated discovery system will detect your repository within 24 hours and start generating metrics automatically.
</p>

<h4>How it works</h4>

<ol>
  <li>Add the badge URL to your repository's README</li>
  <li>Star ⭐ the <code>infra-metrics-insight-renderer</code> repository</li>
  <li>Wait for automatic discovery (runs daily at 02:00 UTC)</li>
  <li>Your metrics badge will be generated and auto-updated</li>
</ol>

<h3 align="center">Manual registration (optional)</h3>

<p>
  For immediate registration or custom configuration, add an entry to
  <a href="targets/targets.yaml"><code>targets/targets.yaml</code></a>. The orchestrator normalizes the identifier into a slug that becomes the SVG filename. Scheduled renders run automatically once the target is listed.
</p>

<p>
  After registration lands in <code>main</code>, trigger the on-demand workflow named <code>render-&lt;slug&gt;.yml</code> to produce the first
  badge artifact immediately.
</p>

<h3 align="center" id="badge-catalogue">Badge catalogue</h3>

<p>
  The published badges are grouped by color so their category is obvious at a glance. Reuse the badges directly from the
  repository to avoid stale snapshots.
</p>

<p align="right"><em><a href="#top">Back to top</a></em></p>

<h4 align="center" id="open-source-badges">🟩 Open-source badges</h4>

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

<p align="right"><em><a href="#top">Back to top</a></em></p>

<h4 align="center" id="private-project-badges">🟦 Private project badges</h4>

<p>
  Private dashboards follow the same embedding rules. Publish badges from this section once private projects are registered.
</p>

<p align="right"><em><a href="#top">Back to top</a></em></p>

<h4 align="center" id="profile-badges">🟪 Profile badges</h4>

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

<p align="right"><em><a href="#top">Back to top</a></em></p>

<h4 align="center" id="color-reference">Color reference</h4>
<ul>
  <li>🟩 Green badges indicate open-source repositories.</li>
  <li>🟦 Blue badges denote private repositories.</li>
  <li>🟪 Purple badges represent GitHub profile dashboards.</li>
</ul>

<p align="right"><em><a href="#top">Back to top</a></em></p>

<h2 align="center" id="imir-cli">IMIR CLI</h2>

<p>
  The <code>imir</code> crate lives in
  <a href="imir"><code>imir</code></a>. It validates the target configuration,
  applies deterministic defaults for filenames, branch names, and time zones, and serializes the normalized targets in JSON form.
  Unit tests cover slug normalization, configuration validation, and duplicate detection to ensure predictable behaviour when new
  targets are added.
</p>

<p align="right"><em><a href="#top">Back to top</a></em></p>

<h2 align="center" id="storage-strategy">Storage strategy</h2>

<p>
  <strong>Current approach:</strong> SVG artifacts are committed directly to the repository under <code>metrics/</code>.
  GitHub Actions workflows render updated metrics on a schedule or when configuration changes, commit them to the main branch, and serve
  them via <code>raw.githubusercontent.com</code>. This approach eliminates the need for separate hosting infrastructure and guarantees
  that badges remain accessible as long as the repository is public.
</p>

<p>
  <strong>Trade-offs:</strong>
</p>
<ul>
  <li><strong>Simplicity</strong> &mdash; no external services, databases, or CDN configuration required. Repository links work immediately.</li>
  <li><strong>Git history noise</strong> &mdash; automated commits for metrics refreshes accumulate in the commit log, though they are clearly
    prefixed with <code>chore(metrics):</code> for easy filtering.</li>
  <li><strong>Version control overhead</strong> &mdash; binary SVG diffs increase repository size over time, but the impact remains negligible
    for typical badge refresh frequencies.</li>
</ul>

<p>
  <strong>Future evolution:</strong> As the number of tracked repositories and refresh frequency grow, migrating to a dedicated database backend
  (PostgreSQL, SQLite, or object storage) paired with a lightweight API server becomes viable. A database-backed approach would eliminate
  commit noise, enable historical metric queries, support versioning, and decouple badge serving from git operations. The migration path is
  straightforward: existing workflows already isolate rendering logic in the <code>imir</code> CLI, so switching the storage layer requires
  only updating the commit step to an API call.
</p>

<p>
  For now, the in-repository strategy prioritizes zero-friction setup and maintenance. When usage patterns justify the additional complexity,
  database storage can be introduced incrementally without disrupting existing badge URLs or workflow triggers.
</p>

<p align="right"><em><a href="#top">Back to top</a></em></p>

<h2 align="center" id="local-development-workflow">Local development workflow</h2>

<p>
  Use <a href="scripts/ci-check.sh"><code>scripts/ci-check.sh</code></a> to run the full validation pipeline locally. The helper script formats the code
  with the nightly toolchain, executes Clippy, builds all targets, runs tests, generates documentation, and invokes <a href="https://crates.io/crates/cargo-audit">cargo audit</a>
  and <a href="https://crates.io/crates/cargo-deny">cargo deny</a> to ensure dependency health. Install
  <a href="https://crates.io/crates/cargo-audit">cargo-audit</a> and <a href="https://crates.io/crates/cargo-deny">cargo-deny</a> beforehand to enable the security checks.
</p>

<p align="right"><em><a href="#top">Back to top</a></em></p>

<h2 align="center" id="release-process">Release process</h2>

<p>
  Tagged releases publish pre-built <code>imir</code> binaries so GitHub Actions workflows can download a pinned CLI without
  rebuilding the crate on every run. Follow this checklist to cut a new release:
</p>

<ol>
  <li>Run <a href="scripts/ci-check.sh"><code>scripts/ci-check.sh</code></a> locally to ensure formatting, linting, tests,
    documentation, <code>cargo audit</code>, and <code>cargo deny</code> all pass before tagging.</li>
  <li>Create an annotated tag (for example, <code>git tag -a v0.1.0</code>) and push it to GitHub.</li>
  <li>Draft a release in the GitHub UI, associate it with the tag, and publish it. Publishing triggers
    <code>.github/workflows/release.yml</code>.</li>
  <li>The workflow builds the CLI for Linux (<code>x86_64-unknown-linux-gnu</code>), packages the binary as
    <code>imir-x86_64-unknown-linux-gnu.tar.gz</code>, and uploads it to the release assets.</li>
  <li>Update downstream workflows to download the archive that matches their runner architecture and unpack the <code>imir</code>
    executable into their workspace.</li>
</ol>

<p>
  Each archive contains only the compiled binary. The release workflow runs on every published release, ensuring updated
  binaries are available immediately after a tag is promoted.
</p>

<p align="right"><em><a href="#top">Back to top</a></em></p>
