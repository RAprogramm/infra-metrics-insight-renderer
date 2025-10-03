//! Command-line interface for the metrics orchestrator binary.
//!
//! The CLI exposes subcommands for normalizing target configuration documents
//! and resolving workflow inputs specific to open-source repository rendering.

use std::{io, path::PathBuf, process};

use clap::{ArgAction, Args, Parser, Subcommand};
use metrics_orchestrator::{load_targets, resolve_open_source_repositories, Error};

/// Command line interface for generating normalized metrics target definitions.
#[derive(Debug, Parser)]
#[command(
    name = "metrics-orchestrator",
    version,
    about = "Normalize metrics renderer targets"
)]
/// Top-level CLI options parsed from user input.
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
/// Supported commands exposed by the CLI.
enum Command {
    /// Normalize targets from a YAML configuration file.
    Targets(TargetsArgs),
    /// Resolve repository inputs for the open-source render workflow.
    #[command(name = "open-source")]
    OpenSource(OpenSourceArgs),
}

#[derive(Debug, Args)]
/// Arguments accepted by the `targets` subcommand.
struct TargetsArgs {
    /// Path to the YAML configuration file describing metrics targets.
    #[arg(long = "config", value_name = "PATH")]
    config: PathBuf,

    /// Output formatted JSON for easier inspection.
    #[arg(long = "pretty", action = ArgAction::SetTrue)]
    pretty: bool,
}

#[derive(Debug, Args)]
/// Arguments accepted by the `open-source` subcommand.
struct OpenSourceArgs {
    /// Raw repositories JSON provided by the workflow input.
    #[arg(long = "input", value_name = "JSON")]
    input: Option<String>,
}

/// Entry point that reports errors and sets the appropriate exit status.
fn main() {
    if let Err(error) = run() {
        eprintln!("{}", error.to_display_string());
        process::exit(1);
    }
}

/// Executes the CLI using parsed arguments.
///
/// # Errors
///
/// Propagates errors originating from configuration loading and normalization.
fn run() -> Result<(), Error> {
    let cli = Cli::parse();

    match cli.command {
        Command::Targets(args) => run_targets(args),
        Command::OpenSource(args) => run_open_source(args),
    }
}

/// Handles the `targets` subcommand by emitting normalized JSON to stdout.
///
/// # Errors
///
/// Returns an [`Error`] when the configuration cannot be loaded or serialized.
fn run_targets(args: TargetsArgs) -> Result<(), Error> {
    let document = load_targets(&args.config)?;

    let stdout = io::stdout();
    let mut handle = stdout.lock();

    if args.pretty {
        serde_json::to_writer_pretty(&mut handle, &document)?;
    } else {
        serde_json::to_writer(&mut handle, &document)?;
    }

    Ok(())
}

/// Handles the `open-source` subcommand by normalizing repository inputs.
///
/// # Errors
///
/// Returns an [`Error`] when repository inputs are invalid or serialization
/// fails.
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
