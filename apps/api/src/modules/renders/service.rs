use tronc::{Error, Result};
use uuid::Uuid;

use super::bundle::{self, JobFile};
use super::spec::{self, Spec};
use super::status::{serving, worker_alive};
use super::types::{Render, RenderRequest, RenderRow, Renders, WORKER_DOWN, columns, is_final};
use crate::extract::db;
use crate::modules::assets::service as assets;
use crate::modules::projects::access::{self, Access};
use crate::modules::projects::error::ProjectError;
use crate::modules::projects::service::fetch;
use crate::state::AppState;

/// One row, whoever asks: callers check access first.
pub async fn row(state: &AppState, id: Uuid) -> Result<RenderRow> {
    sqlx::query_as::<_, RenderRow>(concat!("SELECT ", columns!(), " FROM renders WHERE id = $1"))
        .bind(id)
        .fetch_optional(&state.pool)
        .await
        .map_err(db)?
        .ok_or_else(|| Error::not_found("render not found"))
}

/// The render `id`, for a caller whose access to its project reaches `need`: `404` when they
/// cannot see the project, `403` when they can but may not do this. A posted bundle has no
/// project: it belongs to the account that posted it, and to nobody else.
pub async fn authorized(state: &AppState, user_id: i64, id: Uuid, need: Access) -> Result<RenderRow> {
    let row = row(state, id).await?;
    let have = match row.project_id {
        Some(project) => access::access(state, user_id, project).await?,
        None if row.requested_by == Some(user_id) => Access::Manager,
        None => Access::None,
    };
    match have {
        Access::None => Err(Error::not_found("render not found")),
        have if have < need => Err(Error::forbidden("you may not do this to the render")),
        _ => Ok(row),
    }
}

/// Queues one row, unless the instance already has `RENDER_QUEUE_MAX` renders waiting or
/// running: `429`. `project` is the project and its revision, or `(None, 0)` for a posted bundle.
pub async fn insert(state: &AppState, user_id: i64, project: (Option<Uuid>, i64), spec: &Spec) -> Result<RenderRow> {
    let inserted = sqlx::query_as::<_, RenderRow>(concat!(
        "INSERT INTO renders (project_id, requested_by, rev, width, height, fps, range_start, range_end, frames) \
         SELECT $1, $2, $3, $4, $5, $6, $7, $8, $9 \
         WHERE (SELECT count(*) FROM renders WHERE state IN ('queued', 'running')) < $10 RETURNING ",
        columns!()
    ))
    .bind(project.0)
    .bind(user_id)
    .bind(project.1)
    .bind(spec.width as i32)
    .bind(spec.height as i32)
    .bind(spec.fps as i32)
    .bind(spec.range.map(|range| range.0))
    .bind(spec.range.map(|range| range.1))
    .bind(spec.frames)
    .bind(i64::from(state.limits.render.queue))
    .fetch_optional(&state.pool)
    .await
    .map_err(db)?;
    inserted.ok_or_else(|| Error::rate_limited("too many renders are waiting: try again when one has finished"))
}

/// Forgets a render whose job directory could not be written.
pub async fn discard(state: &AppState, id: Uuid) {
    tokio::fs::remove_dir_all(bundle::dir(&state.data_dir, id)).await.ok();
    sqlx::query("DELETE FROM renders WHERE id = $1").bind(id).execute(&state.pool).await.ok();
}

/// `POST /projects/{id}/renders`: checks the request against the caps, compiles the project as
/// it is now, then queues the job by writing its directory. The worker does the rest.
pub async fn enqueue(
    state: &AppState,
    user_id: i64,
    project: Uuid,
    request: &RenderRequest,
) -> std::result::Result<Render, ProjectError> {
    access::check(state, user_id, project, Access::Editor).await?;
    let source = fetch(state, project).await?;
    if source.trashed_at.is_some() {
        return Err(Error::failed("the project is in the trash").into());
    }
    serving(state).await?;
    let parsed = scene::parse(&source.source).map_err(ProjectError::Invalid)?;
    let size = parsed.config().size;
    let spec = spec::spec(request, (size.width, size.height), parsed.duration(), &state.limits.render)?;
    let plan = bundle::plan(&parsed, &assets::catalog(state, project).await?)?;
    let row = insert(state, user_id, (Some(project), source.rev), &spec).await?;
    if let Err(error) = super::stage::write(&state.data_dir, &plan, &JobFile::of(&row, &spec)).await {
        discard(state, row.id).await;
        return Err(Error::internal("failed to write a render job", error).into());
    }
    state.events.render(row.id);
    Ok(row.into())
}

/// `GET /renders/{id}`: the render, for a caller who may read its project. An unfinished one
/// says so when the worker's heartbeat is stale.
pub async fn read(state: &AppState, user_id: i64, id: Uuid) -> Result<Render> {
    let render = Render::from(authorized(state, user_id, id, Access::Viewer).await?);
    let stalled = !is_final(&render.state) && !worker_alive(&state.data_dir).await;
    Ok(Render { notice: stalled.then_some(WORKER_DOWN), ..render })
}

/// `GET /projects/{id}/renders`: the renders of a project, newest first.
pub async fn list(state: &AppState, user_id: i64, project: Uuid) -> Result<Renders> {
    access::check(state, user_id, project, Access::Viewer).await?;
    let query = concat!("SELECT ", columns!(), " FROM renders WHERE project_id = $1 ORDER BY created_at DESC, id");
    let rows = sqlx::query_as::<_, RenderRow>(query).bind(project).fetch_all(&state.pool).await.map_err(db)?;
    Ok(Renders { renders: rows.into_iter().map(Render::from).collect() })
}

/// `DELETE /renders/{id}`. A render that is waiting or running is asked to stop: the `cancel`
/// file is written and the worker answers `cancelled`; `Some` is the render as it is now. A
/// finished one is removed with its file: `None`.
pub async fn cancel(state: &AppState, user_id: i64, id: Uuid) -> Result<Option<Render>> {
    let row = authorized(state, user_id, id, Access::Editor).await?;
    if !is_final(&row.state) {
        let (data_dir, relative) = (state.data_dir.to_path_buf(), format!("{id}/cancel"));
        let asked = tokio::task::spawn_blocking(move || super::jobfs::touch(&data_dir, &relative)).await;
        asked
            .map_err(|error| Error::internal("failed to ask the worker to stop", error))?
            .map_err(|error| Error::internal("failed to ask the worker to stop", error))?;
        return Ok(Some(row.into()));
    }
    sqlx::query("DELETE FROM renders WHERE id = $1").bind(id).execute(&state.pool).await.map_err(db)?;
    tokio::fs::remove_file(super::status::output(&state.data_dir, id)).await.ok();
    tokio::fs::remove_file(crate::modules::public::poster::path(&state.data_dir, id)).await.ok();
    tokio::fs::remove_dir_all(bundle::dir(&state.data_dir, id)).await.ok();
    Ok(None)
}
