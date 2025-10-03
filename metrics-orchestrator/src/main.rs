use std::path::PathBuf;

use clap::{ArgAction, Parser};
use metrics_orchestrator::{load_targets, Error};

/// Command line interface for generating normalized metrics target definitions.
#[derive(Debug, Parser)]
#[command(
    name = "metrics-orchestrator",
    version,
    about = "Normalize metrics renderer targets"
)]
struct Cli {
    /// Path to the YAML configuration file describing metrics targets.
    #[arg(long = "config", value_name = "PATH")]
    config: PathBuf,

    /// Output formatted JSON for easier inspection.
    #[arg(long = "pretty", action = ArgAction::SetTrue)]
    pretty: bool,
}

fn main() {
    if let Err(error) = run() {
        eprintln!("{}", error.to_display_string());
        std::process::exit(1);
    }
}

fn run() -> Result<(), Error> {
    let cli = Cli::parse();
    let document = load_targets(&cli.config)?;

    let stdout = std::io::stdout();
    let mut handle = stdout.lock();

    if cli.pretty {
        serde_json::to_writer_pretty(&mut handle, &document)?;
    } else {
        serde_json::to_writer(&mut handle, &document)?;
    }

    Ok(())
}
