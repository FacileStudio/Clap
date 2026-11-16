//! `clap new`: a folder with a starter project.

use std::fs;

use serde_json::json;

use crate::cli::NewArgs;
use crate::commands::print_json;
use crate::failure::{Failure, Outcome};
use crate::project::ENTRY;
use crate::starters;
use crate::sync::ASSETS_DIR;
use crate::ui;

/// Creates the folder, its `clap.md` from the chosen starter or the welcome scene, and its
/// `assets/`.
pub fn run(args: &NewArgs) -> Outcome {
    let dir = &args.name;
    let shown = dir.display();
    let template = args.template.as_deref().map(|name| starters::find(name).ok_or(name)).transpose();
    let template = template.map_err(|name| {
        let message = format!("no starter is called `{name}` — run `clap starters` to list them");
        Failure::new("unknown_starter", message)
    })?;
    if dir.exists() {
        return Err(Failure::new("exists", format!("{shown} already exists — pick another name")));
    }
    let title = dir.file_name().map(|name| name.to_string_lossy().into_owned()).unwrap_or_default();
    fs::create_dir_all(dir.join(ASSETS_DIR))?;
    let source = template.map_or_else(|| starters::welcome(&title), |starter| starters::project(starter, &title));
    fs::write(dir.join(ENTRY), source)?;
    if ui::json() {
        print_json(&json!({ "dir": dir, "entry": dir.join(ENTRY), "template": template.map(|starter| starter.name) }));
        return Ok(());
    }
    ui::success(&format!("Created {shown} with {ENTRY} and {ASSETS_DIR}/"));
    ui::hint(&format!("Run `clap serve {shown}` to preview it, `clap render {shown}` for an MP4"));
    Ok(())
}
