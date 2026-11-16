use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tronc::mcp::{Answer, Call, ToolError};

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

/// What `clap_timeline_edit` answers.
#[derive(Serialize, JsonSchema)]
pub struct TimelineOut {
    pub id: String,
    pub title: String,
    /// The project's new revision: name it in the next write.
    pub rev: i64,
    /// How many scenes the project has now.
    pub scenes: usize,
    /// How long the project lasts now, in seconds.
    pub duration: f64,
    /// The ids, as they are now, of the clips the operations changed or added.
    pub clips: Vec<String>,
    /// The page a person opens to see the project.
    pub url: String,
}

/// `clap_timeline_edit`: timeline operations on the source, if the project is still at `rev`.
pub async fn edit(ctx: Context, call: Call, input: TimelineIn) -> Result<Answer<TimelineOut>, ToolError> {
    let project_id = id(&input.project_id).map_err(ProjectError::Api)?;
    let request = EditsRequest { rev: input.rev, ops: input.ops };
    let edited = timeline::edits(&ctx.state, &Author::agent(user(&call)?), project_id, request).await?;
    let out = TimelineOut {
        id: edited.project.id.to_string(),
        title: edited.project.title,
        rev: edited.project.rev,
        scenes: edited.timeline.scenes.len(),
        duration: edited.timeline.duration,
        clips: edited.clips,
        url: ctx.project_url(edited.project.id),
    };
    Ok(out.into())
}
