//! `clap starters`: the ready-made scenes `clap new --template` takes.

use serde_json::json;

use crate::commands::print_json;
use crate::failure::Outcome;
use crate::starters::STARTERS;
use crate::ui;

/// Prints one starter per line: its name, its length, what it is.
pub fn run() -> Outcome {
    if ui::json() {
        print_json(&json!(STARTERS));
        return Ok(());
    }
    let width = STARTERS.iter().map(|starter| starter.name.len()).max().unwrap_or(0);
    for starter in STARTERS {
        println!("{:width$}  {} s  {}", starter.name, starter.seconds, starter.summary);
    }
    Ok(())
}
