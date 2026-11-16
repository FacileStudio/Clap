use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tronc::mcp::{Answer, Call, ToolError};

use super::{Context, user};
use crate::extract::id;
use crate::modules::assets::service;
use crate::modules::projects::error::ProjectError;

/// `clap_list_assets`.
#[derive(Deserialize, JsonSchema)]
pub struct AssetsIn {
    /// The project id.
    pub project_id: String,
}

/// One asset of the project.
#[derive(Serialize, JsonSchema)]
pub struct Item {
    /// The path to write in the source, such as `assets/clip.mp4`.
    pub name: String,
    /// `video`, `audio`, `image` or `other` (a font).
    pub kind: String,
    pub mime: String,
    /// Bytes.
    pub size: i64,
    /// The length in milliseconds, for a video or a sound.
    pub duration_ms: Option<i64>,
    /// Pixels, for a video or a picture.
    pub width: Option<i32>,
    /// Pixels, for a video or a picture.
    pub height: Option<i32>,
    /// Frames a second, for a video.
    pub fps: Option<f64>,
    /// True when a video carries sound.
    pub has_audio: bool,
    /// `probing` while the file is being read, then `ready` or `failed`.
    pub state: String,
}

/// What `clap_list_assets` answers.
#[derive(Serialize, JsonSchema)]
pub struct AssetsOut {
    pub assets: Vec<Item>,
    /// The page a person opens to see the project and upload more.
    pub url: String,
}

/// `clap_list_assets`: the media the project's source may reference.
pub async fn list(ctx: Context, call: Call, input: AssetsIn) -> Result<Answer<AssetsOut>, ToolError> {
    let project_id = id(&input.project_id).map_err(ProjectError::Api)?;
    let listed = service::list(&ctx.state, user(&call)?, project_id).await.map_err(ProjectError::Api)?;
    let item = |asset: service::Asset| Item {
        name: asset.name,
        kind: asset.kind,
        mime: asset.mime,
        size: asset.size,
        duration_ms: asset.duration_ms,
        width: asset.width,
        height: asset.height,
        fps: asset.fps,
        has_audio: asset.has_audio,
        state: asset.state,
    };
    Ok(AssetsOut { assets: listed.assets.into_iter().map(item).collect(), url: ctx.project_url(project_id) }.into())
}
