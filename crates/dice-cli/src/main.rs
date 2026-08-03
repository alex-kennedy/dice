use std::num::NonZeroUsize;
use std::process::ExitCode;

use clap::{Parser, Subcommand};

mod render;
mod simulate;

/// CLI for dice roll distributions, primarily for debugging.
#[derive(Parser)]
#[command(name = "dice", version, about, long_about = None)]
struct Cli {
  #[command(subcommand)]
  command: Command,
}

#[derive(Subcommand)]
enum Command {
  /// Rolls an expression many times and plots the distribution of the results.
  Simulate {
    /// The dice expression to roll, e.g. "2d6 + 4".
    expression: String,

    /// How many rolls to simulate.
    #[arg(short, long, default_value = "100000")]
    runs: NonZeroUsize,
  },
}

fn main() -> ExitCode {
  let cli = Cli::parse();
  let result = match cli.command {
    Command::Simulate { expression, runs } => simulate::run(&expression, runs.get()),
  };
  match result {
    Ok(()) => ExitCode::SUCCESS,
    Err(err) => {
      eprintln!("{err}");
      ExitCode::FAILURE
    }
  }
}
