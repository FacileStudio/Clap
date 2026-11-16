use axum::Router;
use axum::routing::get;

use super::events::events;
use super::handlers::{cancel, create, list, read};
use super::serve::file;
use crate::state::AppState;

/// The render routes, mounted under `/api` behind `require_auth`. `POST` answers `202` with the
/// queued render. `DELETE` asks a waiting or running render to stop (`202`), and removes a
/// finished one with its file (`204`).
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/projects/{id}/renders", get(list).post(create))
        .route("/renders/{id}", get(read).delete(cancel))
        .route("/renders/{id}/events", get(events))
        .route("/renders/{id}/file", get(file))
}
