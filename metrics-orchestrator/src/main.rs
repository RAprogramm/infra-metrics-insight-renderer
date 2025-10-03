use std::{
    io,
    path::{Path, PathBuf},
    process,
};

use clap::{ArgAction, Args, Parser, Subcommand};
use metrics_orchestrator::{load_targets, resolve_open_source_repositories, Error};

/// Command line interface for generating normalized metrics target definitions.
#[derive(Debug, Parser)]
#[command(
    name = "metrics-orchestrator",
    version,
    about = "Normalize metrics renderer targets"
)]
struct Cli {
    /// Optional subcommand invoked by the user.
    #[command(subcommand)]
    command: Option<Command>,

    /// Legacy argument support for the default targets command.
    #[command(flatten)]
    legacy: LegacyTargetsArgs,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Normalize targets from a YAML configuration file.
    Targets(TargetsArgs),
    /// Resolve repository inputs for the open-source render workflow.
    #[command(name = "open-source")]
    OpenSource(OpenSourceArgs),
}

#[derive(Debug, Args)]
struct TargetsArgs {
    /// Path to the YAML configuration file describing metrics targets.
    #[arg(long = "config", value_name = "PATH")]
    config: PathBuf,

    /// Output formatted JSON for easier inspection.
    #[arg(long = "pretty", action = ArgAction::SetTrue)]
    pretty: bool,
}

/// Arguments accepted when the CLI is invoked without a subcommand.
#[derive(Debug, Args, Default)]
struct LegacyTargetsArgs {
    /// Path to the YAML configuration file describing metrics targets.
    #[arg(long = "config", value_name = "PATH")]
    config: Option<PathBuf>,

    /// Output formatted JSON for easier inspection.
    #[arg(long = "pretty", action = ArgAction::SetTrue)]
    pretty: bool,
}

#[derive(Debug, Args)]
struct OpenSourceArgs {
    /// Raw repositories JSON provided by the workflow input.
    #[arg(long = "input", value_name = "JSON")]
    input: Option<String>,
}

fn main() {
    if let Err(error) = run() {
        eprintln!("{}", error.to_display_string());
        process::exit(1);
    }
}

fn run() -> Result<(), Error> {
    let cli = Cli::parse();

    match cli.command {
        Some(Command::Targets(args)) => run_targets(args),
        Some(Command::OpenSource(args)) => run_open_source(args),
        None => run_legacy_targets(&cli.legacy),
    }
}

fn run_targets(args: TargetsArgs) -> Result<(), Error> {
    run_targets_from_path(&args.config, args.pretty)
}

fn run_targets_from_path(path: &Path, pretty: bool) -> Result<(), Error> {
    let document = load_targets(path)?;

    let stdout = io::stdout();
    let mut handle = stdout.lock();

    if pretty {
        serde_json::to_writer_pretty(&mut handle, &document)?;
    } else {
        serde_json::to_writer(&mut handle, &document)?;
    }

    Ok(())
}

fn run_open_source(args: OpenSourceArgs) -> Result<(), Error> {
    let trimmed = args.input.as_deref().map(str::trim).and_then(|value| {
        if value.is_empty() {
            None
        } else {
            Some(value)
        }
    });

    let repositories = resolve_open_source_repositories(trimmed)?;

    let stdout = io::stdout();
    let mut handle = stdout.lock();
    serde_json::to_writer(&mut handle, &repositories)?;

    Ok(())
}

fn run_legacy_targets(args: &LegacyTargetsArgs) -> Result<(), Error> {
    let config = args
        .config
        .as_deref()
        .ok_or_else(|| Error::validation("missing required --config <PATH> argument"))?;

    run_targets_from_path(config, args.pretty)
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use clap::Parser;

    use super::{run_legacy_targets, Cli, LegacyTargetsArgs};

    #[test]
    fn cli_accepts_legacy_targets_invocation() {
        let cli = Cli::try_parse_from(["metrics-orchestrator", "--config", "config.yaml"])
            .expect("failed to parse CLI");

        assert!(cli.command.is_none());
        assert_eq!(cli.legacy.config.as_deref(), Some(Path::new("config.yaml")));
        assert!(!cli.legacy.pretty);
    }

    #[test]
    fn legacy_targets_require_config_path() {
        let args = LegacyTargetsArgs::default();
        let error = run_legacy_targets(&args).expect_err("expected validation error");

        match error {
            metrics_orchestrator::Error::Validation { message, .. } => {
                assert_eq!(message, "missing required --config <PATH> argument");
            }
            other => panic!("unexpected error variant: {other:?}"),
        }
    }
}
