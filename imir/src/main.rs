// SPDX-FileCopyrightText: 2025 RAprogramm <andrey.rozanov.vl@gmail.com>
//
// SPDX-License-Identifier: MIT

//! Command-line interface for the IMIR binary.
//!
//! The CLI exposes subcommands for normalizing target configuration documents
//! and resolving workflow inputs specific to open-source repository rendering.

use std::{
    io,
    path::{Path, PathBuf},
    process,
};

use clap::{ArgAction, Args, Parser, Subcommand};
use imir::{
    DiscoveryConfig, Error, TargetsDocument, detect_impacted_slugs, discover_badge_users,
    discover_stargazer_repositories, generate_badge_assets, gh_pr_create, git_commit_push,
    load_targets, locate_artifact, move_file, normalize_profile_inputs,
    normalize_repository_inputs, resolve_open_source_repositories, sync_targets,
};
use tracing::info;

/// Command line interface for generating normalized metrics target definitions.
#[derive(Debug, Parser,)]
#[command(name = "imir", version, about = "Normalize metrics renderer targets")]
/// Top-level CLI options parsed from user input.
struct Cli
{
    #[command(subcommand)]
    command: Option<Command,>,

    /// Legacy argument support for the default targets command.
    #[command(flatten)]
    legacy: LegacyTargetsArgs,
}

#[derive(Debug, Subcommand,)]
/// Supported commands exposed by the CLI.
enum Command
{
    /// Normalize targets from a YAML configuration file.
    Targets(TargetsArgs,),
    /// Resolve repository inputs for the open-source render workflow.
    #[command(name = "open-source")]
    OpenSource(OpenSourceArgs,),
    /// Generate badge assets for a normalized target.
    Badge(BadgeArgs,),
    /// Discover repositories using IMIR badges.
    Discover(DiscoverArgs,),
    /// Synchronize discovered repositories with targets.yaml.
    Sync(SyncArgs,),
    /// Show contributor activity for the last 30 days.
    Contributors(ContributorsArgs,),
    /// Detect impacted slugs from git changes.
    Slugs(SlugsArgs,),
    /// Locate generated metrics artifacts.
    Artifact(ArtifactArgs,),
    /// Move files with directory creation.
    File(FileArgs,),
    /// Git operations for commits and pushes.
    Git(GitArgs,),
    /// GitHub CLI operations for PRs.
    Gh(GhArgs,),
    /// Render action input normalization.
    Render(RenderArgs,),
}

#[derive(Debug, Args,)]
/// Arguments accepted by the `targets` subcommand.
struct TargetsArgs
{
    /// Path to the YAML configuration file describing metrics targets.
    #[arg(long = "config", value_name = "PATH")]
    config: PathBuf,

    /// Output formatted JSON for easier inspection.
    #[arg(long = "pretty", action = ArgAction::SetTrue)]
    pretty: bool,
}

/// Arguments accepted when the CLI is invoked without a subcommand.
#[derive(Debug, Args, Default,)]
struct LegacyTargetsArgs
{
    /// Path to the YAML configuration file describing metrics targets.
    #[arg(long = "config", value_name = "PATH")]
    config: Option<PathBuf,>,

    /// Output formatted JSON for easier inspection.
    #[arg(long = "pretty", action = ArgAction::SetTrue)]
    pretty: bool,
}

#[derive(Debug, Args,)]
struct OpenSourceArgs
{
    /// Raw repositories JSON provided by the workflow input.
    #[arg(long = "input", value_name = "JSON")]
    input: Option<String,>,
}

#[derive(Debug, Args,)]
struct BadgeArgs
{
    #[command(subcommand)]
    command: BadgeCommand,
}

#[derive(Debug, Subcommand,)]
enum BadgeCommand
{
    /// Materialize deterministic badge assets for a target slug.
    Generate(BadgeGenerateArgs,),
    /// Generate all badge assets in parallel.
    GenerateAll(BadgeGenerateAllArgs,),
}

#[derive(Debug, Args,)]
struct BadgeGenerateArgs
{
    /// Path to the YAML configuration file describing metrics targets.
    #[arg(long = "config", value_name = "PATH")]
    config: PathBuf,

    /// Slug identifying the target to generate badge assets for.
    #[arg(long = "target", value_name = "SLUG")]
    target: String,

    /// Directory that will receive the SVG and manifest artifacts.
    #[arg(long = "output", value_name = "DIR", default_value = "metrics")]
    output: PathBuf,
}

#[derive(Debug, Args,)]
struct BadgeGenerateAllArgs
{
    /// Path to the YAML configuration file describing metrics targets.
    #[arg(long = "config", value_name = "PATH")]
    config: PathBuf,

    /// Directory that will receive the SVG and manifest artifacts.
    #[arg(long = "output", value_name = "DIR", default_value = "metrics")]
    output: PathBuf,
}

#[derive(Debug, Args,)]
struct DiscoverArgs
{
    /// GitHub personal access token for API authentication.
    #[arg(long = "token", env = "GITHUB_TOKEN")]
    token: String,

    /// Discovery source: badge, stargazers, or all.
    #[arg(long = "source", value_name = "SOURCE", default_value = "all")]
    source: String,

    /// Output format (json or yaml).
    #[arg(long = "format", value_name = "FORMAT", default_value = "json")]
    format: String,

    /// Maximum number of pages to fetch from GitHub API.
    #[arg(long = "max-pages", value_name = "COUNT", default_value = "10")]
    max_pages: u32,
}

#[derive(Debug, Args,)]
struct SyncArgs
{
    /// Path to the YAML configuration file to update.
    #[arg(long = "config", value_name = "PATH")]
    config: PathBuf,

    /// GitHub personal access token for API authentication.
    #[arg(long = "token", env = "GITHUB_TOKEN")]
    token: String,

    /// Discovery source: badge, stargazers, or all.
    #[arg(long = "source", value_name = "SOURCE", default_value = "all")]
    source: String,

    /// Maximum number of pages to fetch from GitHub API.
    #[arg(long = "max-pages", value_name = "COUNT", default_value = "10")]
    max_pages: u32,
}

#[derive(Debug, Args,)]
struct ContributorsArgs
{
    /// Repository owner.
    #[arg(long = "owner", value_name = "OWNER")]
    owner: String,

    /// Repository name.
    #[arg(long = "repo", value_name = "REPO")]
    repo: String,

    /// GitHub personal access token for API authentication.
    #[arg(long = "token", env = "GITHUB_TOKEN")]
    token: String,
}

#[derive(Debug, Args,)]
struct SlugsArgs
{
    /// Base git reference for comparison.
    #[arg(long = "base-ref", value_name = "REF", default_value = "")]
    base_ref: String,

    /// Head git reference for comparison.
    #[arg(long = "head-ref", value_name = "REF", default_value = "HEAD")]
    head_ref: String,

    /// Files to check for changes.
    #[arg(long = "files", value_name = "FILES", num_args = 1.., required = true)]
    files: Vec<String,>,

    /// Path to targets configuration.
    #[arg(long = "config", value_name = "PATH")]
    config: PathBuf,

    /// Event name (schedule, push, pull_request).
    #[arg(long = "event", value_name = "EVENT")]
    event: Option<String,>,
}

#[derive(Debug, Args,)]
struct ArtifactArgs
{
    /// Expected filename or relative path.
    #[arg(long = "temp-artifact", value_name = "PATH", required = true)]
    temp_artifact: String,

    /// GitHub workspace directory.
    #[arg(long = "workspace", value_name = "PATH", required = true)]
    workspace: String,
}

#[derive(Debug, Args,)]
struct FileArgs
{
    #[command(subcommand)]
    command: FileCommand,
}

#[derive(Debug, Subcommand,)]
enum FileCommand
{
    /// Move a file from source to destination.
    Move(FileMoveArgs,),
}

#[derive(Debug, Args,)]
struct FileMoveArgs
{
    /// Source file path.
    #[arg(long = "source", value_name = "PATH", required = true)]
    source: String,

    /// Destination file path.
    #[arg(long = "destination", value_name = "PATH", required = true)]
    destination: String,
}

#[derive(Debug, Args,)]
struct GitArgs
{
    #[command(subcommand)]
    command: GitCommand,
}

#[derive(Debug, Subcommand,)]
enum GitCommand
{
    /// Commit and push changes to a branch.
    #[command(name = "commit-push")]
    CommitPush(GitCommitPushArgs,),
}

#[derive(Debug, Args,)]
struct GitCommitPushArgs
{
    /// Target branch name.
    #[arg(long = "branch", value_name = "BRANCH", required = true)]
    branch: String,

    /// File path to add and commit.
    #[arg(long = "path", value_name = "PATH", required = true)]
    path: String,

    /// Commit message.
    #[arg(long = "message", value_name = "MESSAGE", required = true)]
    message: String,
}

#[derive(Debug, Args,)]
struct GhArgs
{
    #[command(subcommand)]
    command: GhCommand,
}

#[derive(Debug, Subcommand,)]
enum GhCommand
{
    /// Create a pull request idempotently.
    #[command(name = "pr-create")]
    PrCreate(GhPrCreateArgs,),
}

#[derive(Debug, Args,)]
struct GhPrCreateArgs
{
    /// Repository in owner/repo format.
    #[arg(long = "repo", value_name = "REPO", required = true)]
    repo: String,

    /// Head branch name.
    #[arg(long = "head", value_name = "BRANCH", required = true)]
    head: String,

    /// Base branch name.
    #[arg(long = "base", value_name = "BRANCH", required = true)]
    base: String,

    /// PR title.
    #[arg(long = "title", value_name = "TITLE", required = true)]
    title: String,

    /// PR body.
    #[arg(long = "body", value_name = "BODY", required = true)]
    body: String,

    /// Labels to add.
    #[arg(long = "labels", value_name = "LABELS", num_args = 1.., required = false)]
    labels: Vec<String,>,

    /// GitHub token.
    #[arg(long = "token", value_name = "TOKEN", required = true)]
    token: String,
}

#[derive(Debug, Args,)]
struct RenderArgs
{
    #[command(subcommand)]
    command: RenderCommand,
}

#[derive(Debug, Subcommand,)]
enum RenderCommand
{
    /// Normalize profile render inputs.
    #[command(name = "normalize-profile")]
    NormalizeProfile(NormalizeProfileArgs,),
    /// Normalize repository render inputs.
    #[command(name = "normalize-repository")]
    NormalizeRepository(NormalizeRepositoryArgs,),
}

#[derive(Debug, Args,)]
struct NormalizeProfileArgs
{
    #[arg(long = "target-user", value_name = "USER", required = true)]
    target_user: String,

    #[arg(long = "branch-name", value_name = "BRANCH")]
    branch_name: Option<String,>,

    #[arg(long = "target-path", value_name = "PATH")]
    target_path: Option<String,>,

    #[arg(long = "temp-artifact", value_name = "PATH")]
    temp_artifact: Option<String,>,

    #[arg(long = "time-zone", value_name = "TZ")]
    time_zone: Option<String,>,

    #[arg(long = "display-name", value_name = "NAME")]
    display_name: Option<String,>,

    #[arg(long = "include-private", value_name = "BOOL")]
    include_private: Option<String,>,
}

#[derive(Debug, Args,)]
struct NormalizeRepositoryArgs
{
    #[arg(long = "target-repo", value_name = "REPO", required = true)]
    target_repo: String,

    #[arg(long = "target-owner", value_name = "OWNER")]
    target_owner: Option<String,>,

    #[arg(long = "github-repo", value_name = "REPO", required = true)]
    github_repo: String,

    #[arg(long = "target-path", value_name = "PATH")]
    target_path: Option<String,>,

    #[arg(long = "temp-artifact", value_name = "PATH")]
    temp_artifact: Option<String,>,

    #[arg(long = "branch-name", value_name = "BRANCH")]
    branch_name: Option<String,>,

    #[arg(long = "contributors-branch", value_name = "BRANCH")]
    contributors_branch: Option<String,>,

    #[arg(long = "time-zone", value_name = "TZ")]
    time_zone: Option<String,>,
}

/// Entry point that reports errors and sets the appropriate exit status.
#[tokio::main]
async fn main()
{
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info",),),
        )
        .with_target(false,)
        .init();

    if let Err(error,) = run().await {
        eprintln!("{}", error.to_display_string());
        process::exit(1,);
    }
}

/// Executes the CLI using parsed arguments.
///
/// # Errors
///
/// Propagates errors originating from configuration loading and normalization.
async fn run() -> Result<(), Error,>
{
    let cli = Cli::parse();

    match cli.command {
        Some(Command::Targets(args,),) => run_targets(args,),
        Some(Command::OpenSource(args,),) => run_open_source(args,),
        Some(Command::Badge(args,),) => run_badge(args,),
        Some(Command::Discover(args,),) => run_discover(args,).await,
        Some(Command::Sync(args,),) => run_sync(args,).await,
        Some(Command::Contributors(args,),) => run_contributors(args,).await,
        Some(Command::Slugs(args,),) => run_slugs(args,),
        Some(Command::Artifact(args,),) => run_artifact(args,),
        Some(Command::File(args,),) => run_file(args,),
        Some(Command::Git(args,),) => run_git(args,),
        Some(Command::Gh(args,),) => run_gh(args,),
        Some(Command::Render(args,),) => run_render(args,),
        None => run_legacy_targets(&cli.legacy,),
    }
}

fn run_targets(args: TargetsArgs,) -> Result<(), Error,>
{
    run_targets_from_path(&args.config, args.pretty,)
}

fn run_targets_from_path(path: &Path, pretty: bool,) -> Result<(), Error,>
{
    let document = load_targets(path,)?;

    let stdout = io::stdout();
    let mut handle = stdout.lock();

    write_targets_document(&mut handle, &document, pretty,)
}

fn write_targets_document<W: io::Write,>(
    writer: &mut W,
    document: &TargetsDocument,
    pretty: bool,
) -> Result<(), Error,>
{
    if pretty {
        serde_json::to_writer_pretty(writer, document,)?;
    } else {
        serde_json::to_writer(writer, document,)?;
    }

    Ok((),)
}

/// Handles the `open-source` subcommand by normalizing repository inputs.
///
/// # Errors
///
/// Returns an [`Error`] when repository inputs are invalid or serialization
/// fails.
fn run_open_source(args: OpenSourceArgs,) -> Result<(), Error,>
{
    let trimmed = args.input.as_deref().map(str::trim,).filter(|value| !value.is_empty(),);

    let repositories = resolve_open_source_repositories(trimmed,)?;

    let stdout = io::stdout();
    let mut handle = stdout.lock();
    serde_json::to_writer(&mut handle, &repositories,)?;

    Ok((),)
}

fn run_legacy_targets(args: &LegacyTargetsArgs,) -> Result<(), Error,>
{
    let config = args
        .config
        .as_deref()
        .ok_or_else(|| Error::validation("missing required --config <PATH> argument",),)?;

    run_targets_from_path(config, args.pretty,)
}

fn run_badge(args: BadgeArgs,) -> Result<(), Error,>
{
    match args.command {
        BadgeCommand::Generate(arguments,) => run_badge_generate(arguments,),
        BadgeCommand::GenerateAll(arguments,) => run_badge_generate_all(arguments,),
    }
}

fn run_badge_generate(args: BadgeGenerateArgs,) -> Result<(), Error,>
{
    let document = load_targets(&args.config,)?;
    let target =
        document.targets.iter().find(|candidate| candidate.slug == args.target,).ok_or_else(
            || Error::validation(format!("target '{}' was not found", args.target),),
        )?;

    generate_badge_assets(target, &args.output,)?;

    Ok((),)
}

fn run_badge_generate_all(args: BadgeGenerateAllArgs,) -> Result<(), Error,>
{
    use rayon::prelude::*;
    use tracing::{debug, info};

    let document = load_targets(&args.config,)?;
    let output_dir = &args.output;

    info!("Generating {} badge assets in parallel", document.targets.len());

    let results: Vec<_,> = document
        .targets
        .par_iter()
        .map(|target| {
            debug!("Generating badge for {}", target.slug);
            generate_badge_assets(target, output_dir,).map(|_| target.slug.clone(),)
        },)
        .collect();

    let mut error_count = 0;
    let mut failed_targets = Vec::new();
    for result in results {
        if let Err(e,) = result {
            eprintln!("Failed to generate badge: {}", e);
            error_count += 1;
            failed_targets.push(e.to_string(),);
        }
    }

    if error_count > 0 {
        return Err(Error::validation(format!("{} badge(s) failed to generate", error_count),),);
    }

    info!("Successfully generated {} badge assets", document.targets.len());
    Ok((),)
}

async fn run_discover(args: DiscoverArgs,) -> Result<(), Error,>
{
    let config = DiscoveryConfig {
        max_pages: args.max_pages,
        ..Default::default()
    };

    info!("Starting repository discovery using source: {}", args.source);
    let repositories = discover_repositories(&args.token, &args.source, &config,).await?;
    info!("Discovered {} repositories", repositories.len());

    let stdout = io::stdout();
    let mut handle = stdout.lock();

    match args.format.as_str() {
        "json" => {
            serde_json::to_writer_pretty(&mut handle, &repositories,)?;
        }
        "yaml" => {
            serde_yaml::to_writer(&mut handle, &repositories,)?;
        }
        format => {
            return Err(Error::validation(format!("unsupported format: {format}"),),);
        }
    }

    Ok((),)
}

async fn discover_repositories(
    token: &str,
    source: &str,
    config: &DiscoveryConfig,
) -> Result<Vec<imir::DiscoveredRepository,>, Error,>
{
    let mut repositories = Vec::new();

    match source {
        "badge" => {
            let badge_repos = discover_badge_users(token, config,)
                .await
                .map_err(|e| Error::service(e.to_string(),),)?;
            repositories.extend(badge_repos,);
        }
        "stargazers" => {
            let star_repos = discover_stargazer_repositories(token, config,)
                .await
                .map_err(|e| Error::service(e.to_string(),),)?;
            repositories.extend(star_repos,);
        }
        "all" => {
            let badge_repos = discover_badge_users(token, config,)
                .await
                .map_err(|e| Error::service(e.to_string(),),)?;
            let star_repos = discover_stargazer_repositories(token, config,)
                .await
                .map_err(|e| Error::service(e.to_string(),),)?;
            repositories.extend(badge_repos,);
            repositories.extend(star_repos,);

            repositories.sort_by(|a, b| {
                a.owner.cmp(&b.owner,).then_with(|| a.repository.cmp(&b.repository,),)
            },);
            repositories.dedup_by(|a, b| a.owner == b.owner && a.repository == b.repository,);
        }
        source => {
            return Err(Error::validation(format!(
                "unsupported source: {source}. Use: badge, stargazers, or all"
            ),),);
        }
    }

    Ok(repositories,)
}

async fn run_sync(args: SyncArgs,) -> Result<(), Error,>
{
    let config = DiscoveryConfig {
        max_pages: args.max_pages,
        ..Default::default()
    };

    info!("Starting sync with source: {}", args.source);
    let repositories = discover_repositories(&args.token, &args.source, &config,).await?;
    info!("Found {} repositories to sync", repositories.len());

    let added =
        sync_targets(&args.config, &repositories,).map_err(|e| Error::service(e.to_string(),),)?;

    if added > 0 {
        info!("Successfully synced {} new repositories to {}", added, args.config.display());
    } else {
        info!("No new repositories to sync");
    }
    println!("Synced {} new repositories to {}", added, args.config.display());

    Ok((),)
}

async fn run_contributors(args: ContributorsArgs,) -> Result<(), Error,>
{
    use imir::{fetch_contributor_activity, retry::RetryConfig};
    use octocrab::Octocrab;

    info!("Fetching contributor activity for {}/{}", args.owner, args.repo);

    let octocrab = Octocrab::builder()
        .personal_token(args.token.clone(),)
        .build()
        .map_err(|e| Error::service(format!("failed to initialize GitHub client: {e}"),),)?;

    let retry_config = RetryConfig::default();
    let contributors =
        fetch_contributor_activity(&octocrab, &args.owner, &args.repo, &retry_config,).await?;

    let json = serde_json::to_string_pretty(&contributors,)
        .map_err(|e| Error::service(format!("failed to serialize contributors: {e}"),),)?;

    println!("{json}");

    Ok((),)
}

fn run_slugs(args: SlugsArgs,) -> Result<(), Error,>
{
    info!(
        "Detecting impacted slugs: base={}, head={}, files={:?}",
        args.base_ref, args.head_ref, args.files
    );

    let document = load_targets(&args.config,)?;
    let all_slugs: Vec<String,> = document.targets.iter().map(|t| t.slug.clone(),).collect();

    let files: Vec<&str,> = args.files.iter().map(|s| s.as_str(),).collect();

    let base_ref = if args.event == Some("schedule".to_string(),) { "" } else { &args.base_ref };

    let result = detect_impacted_slugs(base_ref, &args.head_ref, &files, &all_slugs,)?;

    let json = serde_json::to_string(&result,)
        .map_err(|e| Error::service(format!("failed to serialize result: {e}"),),)?;

    println!("{json}");

    Ok((),)
}

fn run_artifact(args: ArtifactArgs,) -> Result<(), Error,>
{
    info!(
        "Locating artifact: temp={}, workspace={}",
        args.temp_artifact, args.workspace
    );

    let location = locate_artifact(&args.temp_artifact, &args.workspace,)?;

    let json = serde_json::to_string(&location,)
        .map_err(|e| Error::service(format!("failed to serialize location: {e}"),),)?;

    println!("{json}");

    Ok((),)
}

fn run_file(args: FileArgs,) -> Result<(), Error,>
{
    match args.command {
        FileCommand::Move(move_args,) => {
            info!(
                "Moving file: source={}, destination={}",
                move_args.source, move_args.destination
            );

            let result = move_file(&move_args.source, &move_args.destination,)?;

            let json = serde_json::to_string(&result,)
                .map_err(|e| Error::service(format!("failed to serialize result: {e}"),),)?;

            println!("{json}");

            Ok((),)
        },
    }
}

fn run_git(args: GitArgs,) -> Result<(), Error,>
{
    match args.command {
        GitCommand::CommitPush(push_args,) => {
            info!(
                "Committing and pushing: branch={}, path={}, message={}",
                push_args.branch, push_args.path, push_args.message
            );

            let result = git_commit_push(&push_args.branch, &push_args.path, &push_args.message,)?;

            let json = serde_json::to_string(&result,)
                .map_err(|e| Error::service(format!("failed to serialize result: {e}"),),)?;

            println!("{json}");

            Ok((),)
        },
    }
}

fn run_gh(args: GhArgs,) -> Result<(), Error,>
{
    match args.command {
        GhCommand::PrCreate(pr_args,) => {
            info!(
                "Creating PR: repo={}, head={}, base={}",
                pr_args.repo, pr_args.head, pr_args.base
            );

            let label_refs: Vec<&str,> = pr_args.labels.iter().map(|s| s.as_str(),).collect();

            let result = gh_pr_create(
                &pr_args.repo,
                &pr_args.head,
                &pr_args.base,
                &pr_args.title,
                &pr_args.body,
                &label_refs,
                &pr_args.token,
            )?;

            let json = serde_json::to_string(&result,)
                .map_err(|e| Error::service(format!("failed to serialize result: {e}"),),)?;

            println!("{json}");

            Ok((),)
        },
    }
}

fn run_render(args: RenderArgs,) -> Result<(), Error,>
{
    match args.command {
        RenderCommand::NormalizeProfile(profile_args,) => {
            info!("Normalizing profile inputs: user={}", profile_args.target_user);

            let result = normalize_profile_inputs(
                &profile_args.target_user,
                profile_args.branch_name.as_deref(),
                profile_args.target_path.as_deref(),
                profile_args.temp_artifact.as_deref(),
                profile_args.time_zone.as_deref(),
                profile_args.display_name.as_deref(),
                profile_args.include_private.as_deref(),
            )?;

            let json = serde_json::to_string(&result,)
                .map_err(|e| Error::service(format!("failed to serialize result: {e}"),),)?;

            println!("{json}");

            Ok((),)
        },
        RenderCommand::NormalizeRepository(repo_args,) => {
            info!(
                "Normalizing repository inputs: repo={}",
                repo_args.target_repo
            );

            let result = normalize_repository_inputs(
                &repo_args.target_repo,
                repo_args.target_owner.as_deref(),
                &repo_args.github_repo,
                repo_args.target_path.as_deref(),
                repo_args.temp_artifact.as_deref(),
                repo_args.branch_name.as_deref(),
                repo_args.contributors_branch.as_deref(),
                repo_args.time_zone.as_deref(),
            )?;

            let json = serde_json::to_string(&result,)
                .map_err(|e| Error::service(format!("failed to serialize result: {e}"),),)?;

            println!("{json}");

            Ok((),)
        },
    }
}

#[cfg(test)]
mod tests
{
    use std::{fs, io::Cursor, path::Path};

    use clap::Parser;
    use imir::TargetsDocument;
    use tempfile::tempdir;

    use super::{
        Cli, Command, LegacyTargetsArgs, run_badge, run_legacy_targets, write_targets_document,
    };

    #[test]
    fn cli_accepts_legacy_targets_invocation()
    {
        let cli = Cli::try_parse_from([env!("CARGO_PKG_NAME"), "--config", "config.yaml",],)
            .expect("failed to parse CLI",);

        assert!(cli.command.is_none());
        assert_eq!(cli.legacy.config.as_deref(), Some(Path::new("config.yaml")));
        assert!(!cli.legacy.pretty);
    }

    #[test]
    fn legacy_targets_require_config_path()
    {
        let args = LegacyTargetsArgs::default();
        let error = run_legacy_targets(&args,).expect_err("expected validation error",);

        match error {
            imir::Error::Validation {
                message,
            } => {
                assert_eq!(message, "missing required --config <PATH> argument");
            }
            other => panic!("unexpected error variant: {other:?}"),
        }
    }

    #[test]
    fn targets_subcommand_pretty_flag_uses_pretty_writer()
    {
        let cli = Cli::try_parse_from([
            env!("CARGO_PKG_NAME"),
            "targets",
            "--config",
            "config.yaml",
            "--pretty",
        ],)
        .expect("failed to parse CLI",);

        let args = match cli.command.expect("missing targets command",) {
            Command::Targets(args,) => args,
            _ => panic!("unexpected command variant"),
        };
        assert!(args.pretty);

        let document = TargetsDocument {
            targets: Vec::new(),
        };
        let mut buffer = Cursor::new(Vec::new(),);
        write_targets_document(&mut buffer, &document, args.pretty,)
            .expect("failed to serialize targets",);

        let output = String::from_utf8(buffer.into_inner(),).expect("invalid UTF-8",);
        assert_eq!(output, "{\n  \"targets\": []\n}");
    }

    #[test]
    fn legacy_invocation_without_pretty_uses_compact_writer()
    {
        let cli = Cli::try_parse_from([env!("CARGO_PKG_NAME"), "--config", "config.yaml",],)
            .expect("failed to parse CLI",);

        assert!(cli.command.is_none());
        assert!(!cli.legacy.pretty);

        let document = TargetsDocument {
            targets: Vec::new(),
        };
        let mut buffer = Cursor::new(Vec::new(),);
        write_targets_document(&mut buffer, &document, cli.legacy.pretty,)
            .expect("failed to serialize targets",);

        let output = String::from_utf8(buffer.into_inner(),).expect("invalid UTF-8",);
        assert_eq!(output, "{\"targets\":[]}");
    }

    #[test]
    fn badge_generate_writes_assets()
    {
        let temp = tempdir().expect("failed to create tempdir",);
        let config_path = temp.path().join("targets.yaml",);
        let output_dir = temp.path().join("artifacts",);
        let yaml = r"
targets:
  - owner: example
    repository: repo
    type: open_source
    slug: example-repo
";
        fs::write(&config_path, yaml,).expect("failed to write config",);

        let cli = Cli::try_parse_from([
            env!("CARGO_PKG_NAME"),
            "badge",
            "generate",
            "--config",
            config_path.to_str().expect("utf8",),
            "--target",
            "example-repo",
            "--output",
            output_dir.to_str().expect("utf8",),
        ],)
        .expect("failed to parse badge command",);

        let args = match cli.command.expect("missing command",) {
            Command::Badge(arguments,) => arguments,
            other => panic!("unexpected command variant: {other:?}"),
        };

        run_badge(args,).expect("badge generation failed",);

        let svg_path = output_dir.join("example-repo.svg",);
        let manifest_path = output_dir.join("example-repo.json",);
        assert!(svg_path.exists());
        assert!(manifest_path.exists());
    }

    #[test]
    fn badge_generate_reports_missing_target()
    {
        let temp = tempdir().expect("failed to create tempdir",);
        let config_path = temp.path().join("targets.yaml",);
        let yaml = r"
targets:
  - owner: example
    repository: repo
    type: open_source
    slug: existing
";
        fs::write(&config_path, yaml,).expect("failed to write config",);

        let cli = Cli::try_parse_from([
            env!("CARGO_PKG_NAME"),
            "badge",
            "generate",
            "--config",
            config_path.to_str().expect("utf8",),
            "--target",
            "missing",
        ],)
        .expect("failed to parse badge command",);

        let args = match cli.command.expect("missing command",) {
            Command::Badge(arguments,) => arguments,
            other => panic!("unexpected command variant: {other:?}"),
        };

        let error = run_badge(args,).expect_err("expected missing target error",);
        match error {
            imir::Error::Validation {
                message,
            } => {
                assert!(message.contains("target 'missing' was not found"));
            }
            other => panic!("unexpected error variant: {other:?}"),
        }
    }

    #[test]
    fn targets_command_reads_valid_config()
    {
        let temp = tempdir().expect("failed to create tempdir",);
        let config_path = temp.path().join("targets.yaml",);
        let yaml = r"
targets:
  - owner: testuser
    repository: testrepo
    type: open_source
    slug: test-slug
    display_name: Test Repository
";
        fs::write(&config_path, yaml,).expect("failed to write config",);

        let cli = Cli::try_parse_from([
            env!("CARGO_PKG_NAME"),
            "targets",
            "--config",
            config_path.to_str().expect("utf8",),
        ],)
        .expect("failed to parse targets command",);

        match cli.command.expect("missing command",) {
            Command::Targets(args,) => {
                assert_eq!(args.config, config_path);
                assert!(!args.pretty);
            }
            other => panic!("unexpected command variant: {other:?}"),
        }
    }

    #[test]
    fn targets_command_reports_missing_file()
    {
        let temp = tempdir().expect("failed to create tempdir",);
        let nonexistent = temp.path().join("nonexistent.yaml",);

        let cli = Cli::try_parse_from([
            env!("CARGO_PKG_NAME"),
            "targets",
            "--config",
            nonexistent.to_str().expect("utf8",),
        ],)
        .expect("failed to parse targets command",);

        let args = match cli.command.expect("missing command",) {
            Command::Targets(args,) => args,
            other => panic!("unexpected command variant: {other:?}"),
        };

        let result = super::run_targets(args,);
        assert!(result.is_err(), "should fail for missing file",);
    }

    #[test]
    fn targets_command_reports_invalid_yaml()
    {
        let temp = tempdir().expect("failed to create tempdir",);
        let config_path = temp.path().join("invalid.yaml",);
        fs::write(&config_path, "invalid: [yaml: syntax",).expect("failed to write config",);

        let cli = Cli::try_parse_from([
            env!("CARGO_PKG_NAME"),
            "targets",
            "--config",
            config_path.to_str().expect("utf8",),
        ],)
        .expect("failed to parse targets command",);

        let args = match cli.command.expect("missing command",) {
            Command::Targets(args,) => args,
            other => panic!("unexpected command variant: {other:?}"),
        };

        let result = super::run_targets(args,);
        assert!(result.is_err(), "should fail for invalid YAML",);
    }

    #[test]
    fn discover_command_parses_all_flags()
    {
        let cli = Cli::try_parse_from([
            env!("CARGO_PKG_NAME"),
            "discover",
            "--token",
            "test_token",
            "--source",
            "badge",
            "--format",
            "yaml",
            "--max-pages",
            "5",
        ],)
        .expect("failed to parse discover command",);

        match cli.command.expect("missing command",) {
            Command::Discover(args,) => {
                assert_eq!(args.token, "test_token");
                assert_eq!(args.source, "badge");
                assert_eq!(args.format, "yaml");
                assert_eq!(args.max_pages, 5);
            }
            other => panic!("unexpected command variant: {other:?}"),
        }
    }

    #[test]
    fn sync_command_parses_all_flags()
    {
        let temp = tempdir().expect("failed to create tempdir",);
        let config_path = temp.path().join("targets.yaml",);

        let cli = Cli::try_parse_from([
            env!("CARGO_PKG_NAME"),
            "sync",
            "--config",
            config_path.to_str().expect("utf8",),
            "--token",
            "test_token",
            "--source",
            "stargazers",
            "--max-pages",
            "3",
        ],)
        .expect("failed to parse sync command",);

        match cli.command.expect("missing command",) {
            Command::Sync(args,) => {
                assert_eq!(args.config, config_path);
                assert_eq!(args.token, "test_token");
                assert_eq!(args.source, "stargazers");
                assert_eq!(args.max_pages, 3);
            }
            other => panic!("unexpected command variant: {other:?}"),
        }
    }

    #[test]
    fn open_source_command_handles_empty_input()
    {
        let cli = Cli::try_parse_from([env!("CARGO_PKG_NAME"), "open-source", "--input", "",],)
            .expect("failed to parse open-source command",);

        match cli.command.expect("missing command",) {
            Command::OpenSource(args,) => {
                assert_eq!(args.input, Some(String::new()));
            }
            other => panic!("unexpected command variant: {other:?}"),
        }
    }

    #[test]
    fn open_source_command_parses_valid_json()
    {
        let json_input = r#"[{"owner":"user1","repo":"repo1"},{"owner":"user2","repo":"repo2"}]"#;

        let cli =
            Cli::try_parse_from([env!("CARGO_PKG_NAME"), "open-source", "--input", json_input,],)
                .expect("failed to parse open-source command",);

        match cli.command.expect("missing command",) {
            Command::OpenSource(args,) => {
                assert_eq!(args.input, Some(json_input.to_string()));
            }
            other => panic!("unexpected command variant: {other:?}"),
        }
    }

    #[test]
    fn badge_generate_uses_default_output_dir()
    {
        let temp = tempdir().expect("failed to create tempdir",);
        let config_path = temp.path().join("targets.yaml",);
        let yaml = r"
targets:
  - owner: example
    type: profile
    slug: example-profile
";
        fs::write(&config_path, yaml,).expect("failed to write config",);

        let cli = Cli::try_parse_from([
            env!("CARGO_PKG_NAME"),
            "badge",
            "generate",
            "--config",
            config_path.to_str().expect("utf8",),
            "--target",
            "example-profile",
        ],)
        .expect("failed to parse badge command",);

        let args = match cli.command.expect("missing command",) {
            Command::Badge(arguments,) => arguments,
            other => panic!("unexpected command variant: {other:?}"),
        };

        match args.command {
            super::BadgeCommand::Generate(gen_args,) => {
                assert_eq!(gen_args.output, Path::new("metrics"));
            }
            super::BadgeCommand::GenerateAll(_,) => {
                panic!("unexpected generate-all command in this test");
            }
        }
    }
}
