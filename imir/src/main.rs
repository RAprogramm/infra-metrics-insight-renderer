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
use imir::{load_targets, resolve_open_source_repositories, Error, TargetsDocument};

/// Command line interface for generating normalized metrics target definitions.
#[derive(Debug, Parser)]
#[command(name = "imir", version, about = "Normalize metrics renderer targets")]
/// Top-level CLI options parsed from user input.
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,

    /// Legacy argument support for the default targets command.
    #[command(flatten)]
    legacy: LegacyTargetsArgs,
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

    write_targets_document(&mut handle, &document, pretty)
}

fn write_targets_document<W: io::Write>(
    writer: &mut W,
    document: &TargetsDocument,
    pretty: bool,
) -> Result<(), Error> {
    if pretty {
        serde_json::to_writer_pretty(writer, document)?;
    } else {
        serde_json::to_writer(writer, document)?;
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
    let trimmed = args
        .input
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty());

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
    use std::{io::Cursor, path::Path};

    use clap::Parser;

    use imir::TargetsDocument;

    use super::{run_legacy_targets, write_targets_document, Cli, Command, LegacyTargetsArgs};

    #[test]
    fn cli_accepts_legacy_targets_invocation() {
        let cli = Cli::try_parse_from([env!("CARGO_PKG_NAME"), "--config", "config.yaml"])
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
            imir::Error::Validation { message } => {
                assert_eq!(message, "missing required --config <PATH> argument");
            }
            other => panic!("unexpected error variant: {other:?}"),
        }
    }

    #[test]
    fn targets_subcommand_pretty_flag_uses_pretty_writer() {
        let cli = Cli::try_parse_from([
            env!("CARGO_PKG_NAME"),
            "targets",
            "--config",
            "config.yaml",
            "--pretty",
        ])
        .expect("failed to parse CLI");

        let args = match cli.command.expect("missing targets command") {
            Command::Targets(args) => args,
            _ => panic!("unexpected command variant"),
        };
        assert!(args.pretty);

        let document = TargetsDocument {
            targets: Vec::new(),
        };
        let mut buffer = Cursor::new(Vec::new());
        write_targets_document(&mut buffer, &document, args.pretty)
            .expect("failed to serialize targets");

        let output = String::from_utf8(buffer.into_inner()).expect("invalid UTF-8");
        assert_eq!(output, "{\n  \"targets\": []\n}");
    }

    #[test]
    fn legacy_invocation_without_pretty_uses_compact_writer() {
        let cli = Cli::try_parse_from([env!("CARGO_PKG_NAME"), "--config", "config.yaml"])
            .expect("failed to parse CLI");

        assert!(cli.command.is_none());
        assert!(!cli.legacy.pretty);

        let document = TargetsDocument {
            targets: Vec::new(),
        };
        let mut buffer = Cursor::new(Vec::new());
        write_targets_document(&mut buffer, &document, cli.legacy.pretty)
            .expect("failed to serialize targets");

        let output = String::from_utf8(buffer.into_inner()).expect("invalid UTF-8");
        assert_eq!(output, "{\"targets\":[]}");
    }
}
