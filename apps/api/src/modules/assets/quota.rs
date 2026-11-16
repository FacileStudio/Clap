use tronc::{Error, Result};
use uuid::Uuid;

use crate::extract::db;
use crate::state::AppState;

/// The bytes the assets around `project` hold: those of its space, or of its owner's personal
/// projects, without the asset `name` of `project` itself, which an upload is about to replace.
pub async fn held(state: &AppState, project: Uuid, name: &str) -> Result<u64> {
    let held: i64 = sqlx::query_scalar(
        "SELECT coalesce(sum(a.size), 0)::bigint FROM assets a \
         JOIN projects p ON p.id = a.project_id JOIN projects me ON me.id = $1 \
         WHERE ((me.space_id IS NOT NULL AND p.space_id = me.space_id) \
             OR (me.space_id IS NULL AND p.space_id IS NULL AND p.owner_id = me.owner_id)) \
           AND NOT (a.project_id = $1 AND a.name = $2)",
    )
    .bind(project)
    .bind(name)
    .fetch_one(&state.pool)
    .await
    .map_err(db)?;
    Ok(held.max(0) as u64)
}

/// Refuses with `413` unless `size` more bytes fit under the space cap.
pub async fn room(state: &AppState, project: Uuid, name: &str, size: u64) -> Result<()> {
    let cap = state.limits.space_assets;
    match held(state, project, name).await?.saturating_add(size) > cap {
        true => Err(Error::too_large(format!("the assets of a space hold at most {} MiB together", cap >> 20))),
        false => Ok(()),
    }
}

/// The name the in-flight book reserves bytes under for `project`: its space, or its owner.
pub async fn scope(state: &AppState, project: Uuid) -> Result<String> {
    sqlx::query_scalar("SELECT coalesce('space:' || space_id, 'user:' || owner_id::text) FROM projects WHERE id = $1")
        .bind(project)
        .fetch_optional(&state.pool)
        .await
        .map_err(db)?
        .ok_or_else(|| Error::not_found("project not found"))
}

/// The largest asset of `project`: the most an upload into it may be replacing, so the most it
/// may use past what its space has left until its name is known.
pub async fn replaceable(state: &AppState, project: Uuid) -> Result<u64> {
    let largest: i64 = sqlx::query_scalar("SELECT coalesce(max(size), 0)::bigint FROM assets WHERE project_id = $1")
        .bind(project)
        .fetch_one(&state.pool)
        .await
        .map_err(db)?;
    Ok(largest.max(0) as u64)
}
