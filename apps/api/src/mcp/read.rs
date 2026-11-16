use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use time::format_description::well_known::Rfc3339;
use tronc::mcp::{Answer, Call, RESULT_BUDGET, ToolError, clip};

use super::page::{Listed, Window, window};
use super::{Context, user};
use crate::extract::id;
use crate::modules::assets::service as assets;
use crate::modules::projects::error::ProjectError;
use crate::modules::projects::service;
use crate::modules::projects::types::ListQuery;

/// `clap_list_projects`.
#[derive(Deserialize, JsonSchema)]
pub struct ListIn {
    /// One space; every project you reach, personal ones included, when absent.
    pub space_id: Option<String>,
    /// True lists the trash instead of the live projects.
    pub trashed: Option<bool>,
    /// The `next_cursor` of the previous call; the first projects when absent.
    pub cursor: Option<String>,
    /// How many projects, 1 to 200; 50 when absent.
    pub limit: Option<usize>,
}

/// One project of the list.
#[derive(Serialize, JsonSchema)]
pub struct Item {
    pub id: String,
    pub title: String,
    /// The space the project is in; absent for a personal project.
    pub space_id: Option<String>,
    pub rev: i64,
    /// RFC 3339.
    pub updated_at: String,
    /// The page a person opens to see the project.
    pub url: String,
}

/// `clap_list_projects`: the projects the caller reaches, newest first, a window at a time.
pub async fn list(ctx: Context, call: Call, input: ListIn) -> Result<Answer<Listed<Item>>, ToolError> {
    let query = ListQuery { space_id: input.space_id, trashed: input.trashed.unwrap_or(false) };
    let entries = service::list(&ctx.state, user(&call)?, &query).await.map_err(ProjectError::Api)?;
    let projects = entries
        .into_iter()
        .map(|entry| Item {
            id: entry.id.to_string(),
            title: entry.title,
            space_id: entry.space_id,
            rev: entry.rev,
            updated_at: entry.updated_at.format(&Rfc3339).unwrap_or_default(),
            url: ctx.project_url(entry.id),
        })
        .collect();
    Ok(window(projects, Window { cursor: input.cursor, limit: input.limit })?.into())
}

/// `clap_get_project`.
#[derive(Deserialize, JsonSchema)]
pub struct GetIn {
    /// The project id.
    pub project_id: String,
}

/// What `clap_get_project` answers.
#[derive(Serialize, JsonSchema)]
pub struct GetOut {
    pub id: String,
    pub title: String,
    /// The space the project is in; absent for a personal project.
    pub space_id: Option<String>,
    /// The revision to name in clap_write_project or clap_edit_project.
    pub rev: i64,
    /// The whole project, in the scene format.
    pub source: String,
    /// True when the source was cut to fit: never write it back whole, use clap_edit_project.
    pub truncated: bool,
    /// How many scenes the project has.
    pub scenes: usize,
    /// How long the project lasts, in seconds.
    pub duration: f64,
    /// The asset names the source may load, such as `assets/clip.mp4`; clap_list_assets says more.
    pub assets: Vec<String>,
    /// True when the project is in the trash.
    pub trashed: bool,
    /// The page a person opens to see the project.
    pub url: String,
}

/// `clap_get_project`: the source and the `rev` every write must name.
pub async fn get(ctx: Context, call: Call, input: GetIn) -> Result<Answer<GetOut>, ToolError> {
    let project_id = id(&input.project_id).map_err(ProjectError::Api)?;
    let project = service::read(&ctx.state, user(&call)?, project_id).await.map_err(ProjectError::Api)?;
    let catalog = assets::catalog(&ctx.state, project_id).await.map_err(ProjectError::Api)?;
    let names: Vec<String> = catalog.into_iter().map(|asset| asset.name).collect();
    let parsed = clap_timeline::parse(&project.source).ok();
    let (source, truncated) = clip(&project.source, RESULT_BUDGET);
    let out = GetOut {
        id: project.id.to_string(),
        title: project.title,
        space_id: project.space_id,
        rev: project.rev,
        source: source.to_owned(),
        truncated,
        scenes: parsed.as_ref().map_or(0, |parsed| parsed.scenes().len()),
        duration: parsed.as_ref().map_or(0.0, scene::Project::duration),
        assets: names,
        trashed: project.trashed_at.is_some(),
        url: ctx.project_url(project.id),
    };
    Ok(out.into())
}
