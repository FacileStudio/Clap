//! `clap list`: the projects the account can open.

use serde_json::json;

use crate::cli::{Globals, ListArgs};
use crate::commands::{print_json, session};
use crate::failure::Outcome;
use crate::ui;

/// Prints one project per line: its id, then its title.
pub fn run(args: &ListArgs, globals: &Globals) -> Outcome {
    let projects = session(globals.server.as_deref())?.projects()?;
    if ui::json() {
        print_json(&json!(projects));
    } else if projects.is_empty() && !args.quiet {
        println!("No projects");
    }
    let width = projects.iter().map(|project| project.id.len()).max().unwrap_or(0);
    for project in projects.iter().filter(|_| !ui::json()) {
        match args.quiet {
            true => println!("{}", project.id),
            false => println!("{:width$}  {}", project.id, project.title),
        }
    }
    Ok(())
}
