//! Change events: one in-process broadcast, read by the SSE route of each open editor and of
//! each watched render. It lives in this process, so the app runs as one replica.

mod router;

use tokio::sync::broadcast;
use uuid::Uuid;

pub use router::routes;

/// How many events wait for a slow reader before it is told it lagged.
pub const BUFFER: usize = 256;

/// What happened.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    /// A project changed: it is now at `rev`.
    Changed {
        /// The project.
        project: Uuid,
        /// The revision it is at.
        rev: i64,
    },
    /// An asset of a project moved to `state`: `probing`, `ready`, `failed` or `deleted`.
    Asset {
        /// The project.
        project: Uuid,
        /// The asset name.
        name: String,
        /// The state it is in now.
        state: &'static str,
    },
    /// A render's row changed.
    Render {
        /// The render.
        render: Uuid,
    },
}

/// The broadcast every write tells and every open stream listens to.
#[derive(Debug, Clone)]
pub struct Events {
    sender: broadcast::Sender<Event>,
}

impl Default for Events {
    fn default() -> Self {
        Events { sender: broadcast::channel(BUFFER).0 }
    }
}

impl Events {
    /// Tells every listener that `project` is at `rev`. Nobody listening is not an error.
    pub fn changed(&self, project: Uuid, rev: i64) {
        self.sender.send(Event::Changed { project, rev }).ok();
    }

    /// Tells every listener that the asset `name` of `project` is in `state`.
    pub fn asset(&self, project: Uuid, name: &str, state: &'static str) {
        self.sender.send(Event::Asset { project, name: name.to_owned(), state }).ok();
    }

    /// Tells every listener that `render` moved.
    pub fn render(&self, render: Uuid) {
        self.sender.send(Event::Render { render }).ok();
    }

    /// A receiver of every event told from now on.
    pub fn listen(&self) -> broadcast::Receiver<Event> {
        self.sender.subscribe()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn a_listener_hears_what_is_told_after_it_subscribed() {
        let events = Events::default();
        let project = Uuid::new_v4();
        events.changed(project, 1);
        let mut listener = events.listen();
        events.changed(project, 2);
        events.asset(project, "a.mp4", "ready");
        assert_eq!(listener.recv().await.unwrap(), Event::Changed { project, rev: 2 });
        assert_eq!(listener.recv().await.unwrap(), Event::Asset { project, name: "a.mp4".to_owned(), state: "ready" });
    }
}
