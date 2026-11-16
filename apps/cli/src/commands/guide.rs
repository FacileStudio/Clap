//! `clap guide`: the format reference, for people and for agents.

use serde_json::json;

use crate::commands::print_json;
use crate::failure::Outcome;
use crate::ui;

/// What the command line adds to the guide the server gives: its own commands.
pub const CLI_GUIDE: &str = include_str!("guide.md");

/// The guide: the scene reference, Clap's section, then the command line's.
pub fn text() -> String {
    format!("{}{CLI_GUIDE}", clap_timeline::format_guide())
}

/// Prints the guide.
pub fn run() -> Outcome {
    if ui::json() {
        print_json(&json!({ "guide": text() }));
    } else {
        print!("{}", text());
    }
    Ok(())
}
