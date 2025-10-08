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
    DiscoveryConfig, Error, TargetsDocument, discover_badge_users,
    discover_stargazer_repositories, generate_badge_assets, load_targets,
    resolve_open_source_repositories, sync_targets,
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

    /// Badge URL pattern to search for.
    #[arg(
        long = "badge-pattern",
        value_name = "PATTERN",
        default_value = "RAprogramm/infra-metrics-insight-renderer"
    )]
    badge_pattern: String,

    /// Metrics path pattern to search for.
    #[arg(long = "metrics-pattern", value_name = "PATTERN", default_value = "/metrics/")]
    metrics_pattern: String,
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

    /// Badge URL pattern to search for.
    #[arg(
        long = "badge-pattern",
        value_name = "PATTERN",
        default_value = "RAprogramm/infra-metrics-insight-renderer"
    )]
    badge_pattern: String,

    /// Metrics path pattern to search for.
    #[arg(long = "metrics-pattern", value_name = "PATTERN", default_value = "/metrics/")]
    metrics_pattern: String,
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
    }
}

fn run_badge_generate(args: BadgeGenerateArgs,) -> Result<(), Error,>
{
    let document = load_targets(&args.config,)?;
    let slug = args.target.as_str();
    let target = document
        .targets
        .iter()
        .find(|candidate| candidate.slug.as_str() == slug,)
        .ok_or_else(|| Error::validation(format!("target '{slug}' was not found",),),)?;

    generate_badge_assets(target, &args.output,)?;

    Ok((),)
}

async fn run_discover(args: DiscoverArgs,) -> Result<(), Error,>
{
    let config = DiscoveryConfig {
        max_pages:            args.max_pages,
        badge_url_pattern:    args.badge_pattern.clone(),
        metrics_path_pattern: args.metrics_pattern.clone(),
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
        max_pages:            args.max_pages,
        badge_url_pattern:    args.badge_pattern.clone(),
        metrics_path_pattern: args.metrics_pattern.clone(),
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
        let yaml = r#"
targets:
  - owner: example
    repository: repo
    type: open_source
    slug: example-repo
"#;
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
        let yaml = r#"
targets:
  - owner: example
    repository: repo
    type: open_source
    slug: existing
"#;
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
}
