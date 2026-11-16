//! `/mcp`: eleven curated tools over the projects, their assets and their renders,
//! `clap_delete_project` behind a confirmation.

mod assets;
mod delete;
pub mod guide;
mod page;
mod read;
mod render;
mod timeline;
mod tools;
mod write;

use std::collections::HashMap;
use std::sync::Arc;

use tronc::mcp::{App, AppConfig, AppSpec, Call, ToolError};
use uuid::Uuid;

use crate::state::AppState;

pub use delete::KIND;

const INSTRUCTIONS: &str = "Clap holds video projects. A project is one text file: scenes split by `---` lines, \
each an HTML composition whose clips carry data-start and data-duration, rendered to MP4. Call \
clap_format_guide before you write your first project. Every write names the rev it read: read the project \
with clap_get_project, then write with that rev; a stale rev is refused and nothing changes. Prefer \
clap_edit_project for a small change, clap_timeline_edit to move, trim or split clips, and clap_write_project \
to rewrite the whole project. Media are uploaded by people: clap_list_assets lists the files a source may \
reference. clap_render queues an MP4 and clap_render_status follows it.";

/// What every tool shares: the app state and the origin project links start with.
#[derive(Clone)]
pub struct Context {
    /// The app state.
    pub state: AppState,
    /// The origin of the MCP resource, such as `https://clap.facile.studio`.
    pub origin: Arc<str>,
}

impl Context {
    /// The page a person opens to see the project: `https://<host>/projects/<id>`.
    pub fn project_url(&self, id: Uuid) -> String {
        format!("{}/projects/{id}", self.origin)
    }
}

/// The porte user id of the caller.
pub fn user(call: &Call) -> Result<i64, ToolError> {
    call.caller.user_id.parse().map_err(|_| ToolError("the token names no local account".to_owned()))
}

/// Clap's MCP server from `cfg`: the eleven tools, and `clap_delete_project` confirmed on the
/// client's `/pending` page. `cfg.pending` must be set.
pub fn app(state: &AppState, cfg: AppConfig) -> Result<App, String> {
    let spec = AppSpec {
        name: "clap".to_owned(),
        title: "Clap".to_owned(),
        instructions: INSTRUCTIONS.to_owned(),
        kinds: HashMap::from([(KIND.to_owned(), delete::kind(state))]),
        ..AppSpec::default()
    };
    let mut app = App::new(cfg, spec)?;
    let confirmations = app.confirmations.clone().ok_or("clap: clap_delete_project needs confirmations")?;
    let context = Context { state: state.clone(), origin: Arc::from(app.url.as_str()) };
    app.options.tools = tools::all(&context, &confirmations);
    Ok(app)
}
