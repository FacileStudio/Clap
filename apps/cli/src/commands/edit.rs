//! `clap edit`: timeline operations applied to the project file, as minimal text patches.

use std::fs;
use std::io::Read;

use clap_timeline::Op;
use serde_json::{Value, json};

use crate::cli::EditArgs;
use crate::commands::print_json;
use crate::failure::{Failure, Outcome};
use crate::project::Folder;
use crate::ui;

fn bad(detail: impl std::fmt::Display) -> Failure {
    let hint = r#"pass a JSON array such as '[{"op":"move_clip","clip":"title","start":1.5}]', or @file"#;
    Failure::new("bad_ops", format!("cannot read the operations ({detail}) — {hint}"))
}

fn written(argument: &str) -> Outcome<String> {
    match argument.strip_prefix('@') {
        Some("-") => {
            let mut text = String::new();
            std::io::stdin().read_to_string(&mut text)?;
            Ok(text)
        }
        Some(file) => fs::read_to_string(file).map_err(|error| bad(format!("{file}: {error}"))),
        None => Ok(argument.to_owned()),
    }
}

/// Reads what `--ops` was given: a JSON array of operations, or one operation.
pub fn ops(argument: &str) -> Outcome<Vec<Op>> {
    let value: Value = serde_json::from_str(&written(argument)?).map_err(bad)?;
    let list = if value.is_array() { value } else { json!([value]) };
    serde_json::from_value(list).map_err(bad)
}

/// Applies the operations in order and writes the file once, when all of them applied. A
/// refused operation changes nothing and names its reason.
pub fn run(args: &EditArgs) -> Outcome {
    let folder = Folder::locate(args.dir.as_deref())?;
    let ops = ops(&args.ops)?;
    let before = folder.source()?;
    let after = clap_timeline::apply(&before, &ops).map_err(|error| Failure::new(error.code(), error.to_string()))?;
    let changed = after != before;
    if changed {
        fs::write(&folder.entry, &after)?;
    }
    if ui::json() {
        print_json(&json!({ "file": folder.label(), "ops": ops.len(), "changed": changed }));
        return Ok(());
    }
    let count = if ops.len() == 1 { "1 operation".to_owned() } else { format!("{} operations", ops.len()) };
    match changed {
        true => ui::success(&format!("Applied {count} to {}", folder.label())),
        false => ui::success(&format!("{} already holds the result of {count}", folder.label())),
    }
    Ok(())
}
