use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use tronc::{Error, Result};
use uuid::Uuid;

use crate::extract::db;
use crate::modules::projects::access::{self, Access};
use crate::modules::projects::service::fetch;
use crate::state::AppState;

/// The most markers one project holds.
pub const MAX_MARKERS: i64 = 500;

/// The longest label, in characters.
pub const MAX_LABEL: usize = 200;

/// One marker: a project time in seconds and its label, empty when it has none.
#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct Marker {
    pub id: Uuid,
    pub t: f64,
    pub label: String,
    #[serde(with = "time::serde::rfc3339::option")]
    pub created_at: Option<OffsetDateTime>,
}

/// What `GET /projects/{id}/markers` answers.
#[derive(Debug, Serialize)]
pub struct Markers {
    pub markers: Vec<Marker>,
}

/// `POST /projects/{id}/markers`.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NewMarker {
    pub t: f64,
    #[serde(default)]
    pub label: Option<String>,
}

/// `PATCH /projects/{id}/markers/{marker}`: a field left out keeps its value.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Change {
    #[serde(default)]
    pub t: Option<f64>,
    #[serde(default)]
    pub label: Option<String>,
}

fn time(t: f64) -> Result<f64> {
    match t.is_finite() && t >= 0.0 {
        true => Ok(t),
        false => Err(Error::invalid("t is a number of seconds, 0 or more")),
    }
}

fn label(text: &str) -> Result<String> {
    let text = text.trim();
    match text.chars().count() <= MAX_LABEL {
        true => Ok(text.to_owned()),
        false => Err(Error::invalid(format!("label is at most {MAX_LABEL} characters"))),
    }
}

async fn editable(state: &AppState, user_id: i64, project: Uuid) -> Result<()> {
    access::check(state, user_id, project, Access::Editor).await?;
    match fetch(state, project).await?.trashed_at {
        Some(_) => Err(Error::failed("the project is in the trash")),
        None => Ok(()),
    }
}

/// `GET /projects/{id}/markers`: the markers of a project the caller may read, by time.
pub async fn list(state: &AppState, user_id: i64, project: Uuid) -> Result<Markers> {
    access::check(state, user_id, project, Access::Viewer).await?;
    let markers = sqlx::query_as::<_, Marker>(
        "SELECT id, t, label, created_at FROM markers WHERE project_id = $1 ORDER BY t, created_at, id",
    )
    .bind(project)
    .fetch_all(&state.pool)
    .await
    .map_err(db)?;
    Ok(Markers { markers })
}

/// `POST /projects/{id}/markers`, for an editor: adds a marker at `t`. `412` once the project
/// holds [`MAX_MARKERS`].
pub async fn create(state: &AppState, user_id: i64, project: Uuid, body: NewMarker) -> Result<Marker> {
    editable(state, user_id, project).await?;
    let (t, label) = (time(body.t)?, label(body.label.as_deref().unwrap_or_default())?);
    sqlx::query_as::<_, Marker>(
        "INSERT INTO markers (project_id, t, label) SELECT $1, $2, $3 \
         WHERE (SELECT count(*) FROM markers WHERE project_id = $1) < $4 RETURNING id, t, label, created_at",
    )
    .bind(project)
    .bind(t)
    .bind(label)
    .bind(MAX_MARKERS)
    .fetch_optional(&state.pool)
    .await
    .map_err(db)?
    .ok_or_else(|| Error::failed(format!("a project holds at most {MAX_MARKERS} markers: delete one first")))
}

/// `PATCH /projects/{id}/markers/{marker}`, for an editor: moves or renames a marker.
pub async fn change(state: &AppState, user_id: i64, project: Uuid, marker: Uuid, body: Change) -> Result<Marker> {
    editable(state, user_id, project).await?;
    let t = body.t.map(time).transpose()?;
    let label = body.label.as_deref().map(label).transpose()?;
    sqlx::query_as::<_, Marker>(
        "UPDATE markers SET t = coalesce($3, t), label = coalesce($4, label) \
         WHERE id = $2 AND project_id = $1 RETURNING id, t, label, created_at",
    )
    .bind(project)
    .bind(marker)
    .bind(t)
    .bind(label)
    .fetch_optional(&state.pool)
    .await
    .map_err(db)?
    .ok_or_else(|| Error::not_found("marker not found"))
}

/// `DELETE /projects/{id}/markers/{marker}`, for an editor.
pub async fn remove(state: &AppState, user_id: i64, project: Uuid, marker: Uuid) -> Result<()> {
    editable(state, user_id, project).await?;
    let deleted = sqlx::query("DELETE FROM markers WHERE id = $2 AND project_id = $1")
        .bind(project)
        .bind(marker)
        .execute(&state.pool)
        .await
        .map_err(db)?;
    match deleted.rows_affected() {
        0 => Err(Error::not_found("marker not found")),
        _ => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_time_is_finite_and_not_negative_and_a_label_is_short() {
        assert_eq!(time(0.0).unwrap(), 0.0);
        for bad in [-0.5, f64::NAN, f64::INFINITY] {
            assert!(time(bad).is_err(), "{bad}");
        }
        assert_eq!(label("  Drop  ").unwrap(), "Drop");
        assert!(label(&"é".repeat(MAX_LABEL)).is_ok());
        assert!(label(&"é".repeat(MAX_LABEL + 1)).is_err());
    }
}
