use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tronc::mcp::{Answer, Call, ToolError};

use super::{Context, user};
use crate::extract::id;
use crate::modules::projects::edit::{self, Replacement};
use crate::modules::projects::error::ProjectError;
use crate::modules::projects::types::{Author, CreateRequest, Project, SourceRequest};
use crate::modules::projects::{service, write as projects};

/// What a write answers.
#[derive(Serialize, JsonSchema)]
pub struct Written {
    pub id: String,
    pub title: String,
    /// The project's new revision: name it in the next write.
    pub rev: i64,
    /// How many scenes the project has now.
    pub scenes: usize,
    /// The page a person opens to see the project.
    pub url: String,
}

/// The answer of a write that left `project` as it is now.
pub fn written(ctx: &Context, project: Project) -> Answer<Written> {
    let scenes = scene::parse(&project.source).map_or(0, |project| project.scenes().len());
    let url = ctx.project_url(project.id);
    Written { id: project.id.to_string(), title: project.title, rev: project.rev, scenes, url }.into()
}

/// `clap_create_project`.
#[derive(Deserialize, JsonSchema)]
pub struct CreateIn {
    /// The project title, at most 200 characters.
    pub title: String,
    /// The whole project, in the scene format; a one-scene starter when absent.
    pub source: Option<String>,
    /// The space; a personal project when absent.
    pub space_id: Option<String>,
}

/// `clap_create_project`: a new project at rev 1.
pub async fn create(ctx: Context, call: Call, input: CreateIn) -> Result<Answer<Written>, ToolError> {
    let request = CreateRequest { title: input.title, space_id: input.space_id, source: input.source };
    let project = service::create(&ctx.state, &Author::agent(user(&call)?), request).await?;
    Ok(written(&ctx, project))
}

/// `clap_write_project`.
#[derive(Deserialize, JsonSchema)]
pub struct WriteIn {
    /// The project id.
    pub project_id: String,
    /// The whole new source, in the scene format.
    pub source: String,
    /// The rev clap_get_project answered.
    pub rev: i64,
}

/// `clap_write_project`: replaces the whole source if the project is still at `rev`.
pub async fn write(ctx: Context, call: Call, input: WriteIn) -> Result<Answer<Written>, ToolError> {
    let project_id = id(&input.project_id).map_err(ProjectError::Api)?;
    let request = SourceRequest { source: input.source, rev: input.rev };
    let project = projects::save(&ctx.state, &Author::agent(user(&call)?), project_id, request).await?;
    Ok(written(&ctx, project))
}

/// `clap_edit_project`.
#[derive(Deserialize, JsonSchema)]
pub struct EditIn {
    /// The project id.
    pub project_id: String,
    /// The exact text to replace. It must be in the source exactly once.
    pub old_string: String,
    /// The text that takes its place; empty deletes `old_string`.
    pub new_string: String,
    /// The rev clap_get_project answered.
    pub rev: i64,
}

/// `clap_edit_project`: one exact string replacement, if the project is still at `rev`.
pub async fn edit(ctx: Context, call: Call, input: EditIn) -> Result<Answer<Written>, ToolError> {
    let project_id = id(&input.project_id).map_err(ProjectError::Api)?;
    let change = Replacement { old: input.old_string, new: input.new_string, rev: input.rev };
    let project = edit::edit(&ctx.state, &Author::agent(user(&call)?), project_id, change).await?;
    Ok(written(&ctx, project))
}
