use axum::Router;
use axum::routing::{get, post, put};

use super::handlers::{create, edits, list, read, remove, save, update};
use crate::state::AppState;

/// The project routes, mounted under `/api` behind `require_auth`.
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/projects", get(list).post(create))
        .route("/projects/{id}", get(read).patch(update).delete(remove))
        .route("/projects/{id}/source", put(save))
        .route("/projects/{id}/edits", post(edits))
}
