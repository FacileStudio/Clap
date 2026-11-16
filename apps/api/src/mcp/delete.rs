use schemars::JsonSchema;
use serde::Deserialize;
use tronc::mcp::{Action, Answer, Call, Confirmation, Confirmations, Kind, ToolError};
use uuid::Uuid;

use super::{Context, user};
use crate::extract::id;
use crate::modules::projects::error::ProjectError;
use crate::modules::projects::trash;
use crate::state::AppState;

/// The confirmation kind of `clap_delete_project`.
pub const KIND: &str = "clap_delete_project";

const DONE: &str = "The user confirmed and the project is in the trash, where it stays 30 days.";

/// `clap_delete_project`.
#[derive(Deserialize, JsonSchema)]
pub struct DeleteIn {
    /// The project id.
    pub project_id: String,
}

fn target(action: &Action) -> tronc::Result<(i64, Uuid)> {
    let user = action.user_id.parse().map_err(|e| tronc::Error::internal("bad action user", e))?;
    Ok((user, id(&action.target)?))
}

async fn fingerprint(state: AppState, action: Action) -> tronc::Result<String> {
    let (user, project) = target(&action)?;
    let project = trash::managed(&state, user, project).await?;
    Ok(format!("{}:{}", project.rev, project.trashed_at.is_some()))
}

async fn run(state: AppState, action: Action) -> tronc::Result<()> {
    let (user, project) = target(&action)?;
    trash::trash(&state, user, project).await.map(|_| ())
}

/// The confirmation kind that trashes a project once the person agrees. An edit made in between
/// closes the action.
pub fn kind(state: &AppState) -> Kind {
    let (a, b) = (state.clone(), state.clone());
    Kind::new(KIND, move |action| fingerprint(a.clone(), action), move |action| run(b.clone(), action))
}

/// `clap_delete_project`: asks the person, then moves the project to the trash if they agree.
pub async fn delete(
    ctx: Context,
    confirmations: Confirmations,
    call: Call,
    input: DeleteIn,
) -> Result<Answer<Confirmation>, ToolError> {
    if call.retrying() {
        return Ok(confirmations.answered(&call, KIND, DONE).await?.into());
    }
    let project_id = id(&input.project_id).map_err(ProjectError::Api)?;
    let project = trash::managed(&ctx.state, user(&call)?, project_id).await.map_err(ProjectError::Api)?;
    if project.trashed_at.is_some() {
        return Err(ToolError("the project is already in the trash: nothing to do".to_owned()));
    }
    let question = format!("Move the project \"{}\" to the trash?", project.title);
    confirmations.ask(&call, KIND, &project_id.to_string(), &question).await
}
