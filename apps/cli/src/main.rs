//! The `clap` binary: parses the command line and hands it to the library.

use std::process::ExitCode;

use clap::Parser;

fn main() -> ExitCode {
    clap_cli::run(clap_cli::cli::Cli::parse())
}
