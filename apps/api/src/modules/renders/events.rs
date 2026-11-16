use std::convert::Infallible;

use axum::extract::{Path, State};
use axum::response::sse::{Event as Frame, KeepAlive, Sse};
use futures_util::stream::{self, Stream};
use porte::Identity;
use tokio::sync::broadcast::Receiver;
use tokio::sync::broadcast::error::RecvError;
use tronc::Result;
use uuid::Uuid;

use super::service;
use super::types::{Render, is_final};
use crate::extract::id;
use crate::modules::events::Event;
use crate::state::AppState;

struct Feed {
    state: AppState,
    user_id: i64,
    render: Uuid,
    listener: Receiver<Event>,
    first: Option<Render>,
    over: bool,
}

fn status(render: &Render) -> Frame {
    Frame::default().event("status").data(serde_json::to_string(render).unwrap_or_default())
}

async fn heard(feed: &mut Feed) -> bool {
    loop {
        match feed.listener.recv().await {
            Ok(Event::Render { render }) if render == feed.render => return true,
            Ok(_) => {}
            Err(RecvError::Lagged(_)) => return true,
            Err(RecvError::Closed) => return false,
        }
    }
}

async fn next(mut feed: Feed) -> Option<(std::result::Result<Frame, Infallible>, Feed)> {
    if feed.over {
        return None;
    }
    let render = match feed.first.take() {
        Some(render) => render,
        None => {
            heard(&mut feed).await.then_some(())?;
            service::read(&feed.state, feed.user_id, feed.render).await.ok()?
        }
    };
    feed.over = is_final(&render.state);
    Some((Ok(status(&render)), feed))
}

/// `GET /renders/{id}/events`: a server-sent stream of `status` events, each the render as
/// `GET /renders/{id}` answers it. The first says where the render is now; the stream ends after
/// the one that says `done`, `failed` or `cancelled`, or when the reader loses access.
pub async fn events(
    State(state): State<AppState>,
    who: Identity,
    Path(raw): Path<String>,
) -> Result<Sse<impl Stream<Item = std::result::Result<Frame, Infallible>>>> {
    let render = id(&raw)?;
    let listener = state.events.listen();
    let first = Some(service::read(&state, who.user_id, render).await?);
    let feed = Feed { state, user_id: who.user_id, render, listener, first, over: false };
    Ok(Sse::new(stream::unfold(feed, next)).keep_alive(KeepAlive::default()))
}
