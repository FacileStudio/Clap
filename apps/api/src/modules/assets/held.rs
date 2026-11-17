//! The files a pasted clip loads must be files the project holds.

use clap_timeline::Op;
use uuid::Uuid;

use super::service::catalog;
use crate::modules::projects::error::ProjectError;
use crate::state::AppState;

/// Refuses a `paste_clip` whose element loads a file the project does not hold: `422` with
/// `timeline_error: invalid_op`, as the engine refuses a file it cannot name.
pub async fn held(state: &AppState, project: Uuid, ops: &[Op]) -> std::result::Result<(), ProjectError> {
    let pasted = ops.iter().filter_map(|op| match op {
        Op::PasteClip { element, .. } => Some(clap_timeline::asset_names(element)),
        _ => None,
    });
    let wanted: Vec<String> = pasted.flatten().collect();
    if wanted.is_empty() {
        return Ok(());
    }
    let stored = catalog(state, project).await?;
    match wanted.into_iter().find(|name| !stored.iter().any(|asset| &asset.name == name)) {
        Some(missing) => Err(ProjectError::Timeline(clap_timeline::Error::Invalid(format!(
            "the pasted clip loads {missing}, which the project does not hold: upload it first"
        )))),
        None => Ok(()),
    }
}
