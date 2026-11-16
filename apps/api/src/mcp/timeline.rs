use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::Value;
use tronc::mcp::{Answer, Call, ToolError};

use super::write::{Written, written};
use super::{Context, user};
use crate::extract::id;
use crate::modules::projects::error::ProjectError;
use crate::modules::projects::timeline::{self, EditsRequest};
use crate::modules::projects::types::Author;

/// `clap_timeline_edit`.
#[derive(Deserialize, JsonSchema)]
pub struct TimelineIn {
    /// The project id.
    pub project_id: String,
    /// The rev clap_get_project answered.
    pub rev: i64,
    /// The operations, applied in order; each is an object tagged by `op`, such as
    /// `{"op": "move_clip", "clip": "title", "start": 1.5}`. Nothing is saved unless all apply.
    pub ops: Vec<Value>,
}

/// `clap_timeline_edit`: timeline operations on the source, if the project is still at `rev`.
pub async fn edit(ctx: Context, call: Call, input: TimelineIn) -> Result<Answer<Written>, ToolError> {
    let project_id = id(&input.project_id).map_err(ProjectError::Api)?;
    let request = EditsRequest { rev: input.rev, ops: input.ops };
    let project = timeline::edits(&ctx.state, &Author::agent(user(&call)?), project_id, request).await?;
    Ok(written(&ctx, project))
}
