use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tronc::mcp::{Answer, Call, ToolError};

use super::{Context, user};
use crate::extract::id;
use crate::modules::projects::error::ProjectError;
use crate::modules::renders::service;
use crate::modules::renders::types::{Render, RenderRequest};

/// `clap_render`.
#[derive(Deserialize, JsonSchema)]
pub struct RenderIn {
    /// The project id.
    pub project_id: String,
    /// Pixels, even; the project's own width when absent.
    pub width: Option<u32>,
    /// Pixels, even; the project's own height when absent.
    pub height: Option<u32>,
    /// Frames a second; 30 when absent.
    pub fps: Option<u32>,
    /// `[start, end]` in seconds; the whole project when absent.
    pub range: Option<[f64; 2]>,
}

/// What `clap_render` and `clap_render_status` answer.
#[derive(Serialize, JsonSchema)]
pub struct RenderOut {
    /// The render id: pass it to clap_render_status.
    pub id: String,
    /// The project that is rendered; absent for a bundle another app posted.
    pub project_id: Option<String>,
    /// The project revision that is rendered.
    pub rev: i64,
    /// `queued`, `running`, `done`, `failed` or `cancelled`.
    pub state: String,
    /// The frames captured so far.
    pub frame: i64,
    /// The frames the render has in all.
    pub frames: i64,
    /// Why the render failed, when it did.
    pub error: Option<String>,
    /// Where the person downloads the MP4 in their browser, once the render is done.
    pub download_url: Option<String>,
    /// The page a person opens to see the project, or the app when the render has none.
    pub url: String,
}

fn out(ctx: &Context, render: Render) -> Answer<RenderOut> {
    RenderOut {
        download_url: render.download_url.map(|path| format!("{}{path}", ctx.origin)),
        url: render.project_id.map_or_else(|| ctx.origin.to_string(), |project| ctx.project_url(project)),
        id: render.id.to_string(),
        project_id: render.project_id.map(|project| project.to_string()),
        rev: render.rev,
        state: render.state,
        frame: render.frame,
        frames: render.frames,
        error: render.error,
    }
    .into()
}

/// `clap_render`: queues an MP4 of the project as it is saved now.
pub async fn render(ctx: Context, call: Call, input: RenderIn) -> Result<Answer<RenderOut>, ToolError> {
    let project_id = id(&input.project_id).map_err(ProjectError::Api)?;
    let request = RenderRequest { width: input.width, height: input.height, fps: input.fps, range: input.range };
    let queued = service::enqueue(&ctx.state, user(&call)?, project_id, &request).await?;
    Ok(out(&ctx, queued))
}

/// `clap_render_status`.
#[derive(Deserialize, JsonSchema)]
pub struct StatusIn {
    /// The render id clap_render answered.
    pub render_id: String,
}

/// `clap_render_status`: where a render is.
pub async fn status(ctx: Context, call: Call, input: StatusIn) -> Result<Answer<RenderOut>, ToolError> {
    let render_id = id(&input.render_id).map_err(ProjectError::Api)?;
    let found = service::read(&ctx.state, user(&call)?, render_id).await;
    let found = found.map_err(|error| tronc::mcp::failure(error, "clap_render answers render ids."))?;
    Ok(out(&ctx, found))
}
