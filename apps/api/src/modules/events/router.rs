use std::convert::Infallible;

use axum::Router;
use axum::extract::{Path, State};
use axum::response::sse::{Event as Frame, KeepAlive, Sse};
use axum::routing::get;
use futures_util::stream::{self, Stream};
use porte::Identity;
use serde_json::json;
use tokio::sync::broadcast::Receiver;
use tokio::sync::broadcast::error::RecvError;
use tronc::Result;
use uuid::Uuid;

use super::Event;
use crate::extract::id;
use crate::modules::projects::access::{self, Access};
use crate::modules::projects::service::fetch;
use crate::state::AppState;

struct Feed {
    state: AppState,
    user_id: i64,
    project: Uuid,
    listener: Receiver<Event>,
    first: Option<i64>,
}

fn changed(rev: i64) -> Frame {
    Frame::default().event("changed").data(json!({"rev": rev}).to_string())
}

fn frame(event: Event, project: Uuid) -> Option<Frame> {
    match event {
        Event::Changed { project: of, rev } if of == project => Some(changed(rev)),
        Event::Asset { project: of, name, state } if of == project => {
            Some(Frame::default().event("asset").data(json!({"name": name, "state": state}).to_string()))
        }
        _ => None,
    }
}

async fn heard(feed: &mut Feed) -> Option<Frame> {
    loop {
        match feed.listener.recv().await {
            Ok(event) => match frame(event, feed.project) {
                Some(frame) => return Some(frame),
                None => continue,
            },
            Err(RecvError::Lagged(_)) => {
                return fetch(&feed.state, feed.project).await.ok().map(|row| changed(row.rev));
            }
            Err(RecvError::Closed) => return None,
        }
    }
}

async fn next(mut feed: Feed) -> Option<(std::result::Result<Frame, Infallible>, Feed)> {
    if let Some(rev) = feed.first.take() {
        return Some((Ok(changed(rev)), feed));
    }
    let frame = heard(&mut feed).await?;
    let have = access::access(&feed.state, feed.user_id, feed.project).await.ok()?;
    (have >= Access::Viewer).then(|| (Ok(frame), feed))
}

async fn events(
    State(state): State<AppState>,
    who: Identity,
    Path(raw): Path<String>,
) -> Result<Sse<impl Stream<Item = std::result::Result<Frame, Infallible>>>> {
    let project = id(&raw)?;
    access::check(&state, who.user_id, project, Access::Viewer).await?;
    let listener = state.events.listen();
    let first = Some(fetch(&state, project).await?.rev);
    let feed = Feed { state, user_id: who.user_id, project, listener, first };
    Ok(Sse::new(stream::unfold(feed, next)).keep_alive(KeepAlive::default()))
}

/// `GET /projects/{id}/events`, behind `require_auth`: a server-sent stream. `changed` carries
/// `{"rev": n}`; the first one says where the project is now, so a reader that loaded the project
/// before it subscribed misses nothing. `asset` carries `{"name", "state"}` each time an asset is
/// uploaded, probed or deleted. After a lag the stream answers one `changed`: list the assets
/// again then. The stream ends when the reader loses access.
pub fn routes() -> Router<AppState> {
    Router::new().route("/projects/{id}/events", get(events))
}
