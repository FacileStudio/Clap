use std::path::PathBuf;
use std::sync::Arc;

use porte::exchange::Exchanger;
use sqlx::PgPool;
use tokio::sync::Semaphore;
use tronc::mcp::Confirmations;

use crate::limits::Limits;
use crate::modules::assets::tools::Tools;
use crate::modules::assistant::Bulle;
use crate::modules::events::Events;
use crate::modules::frame::FrameKey;

/// What every handler shares.
#[derive(Clone)]
pub struct AppState {
    /// The database.
    pub pool: PgPool,
    /// The Registre issuer: the `provider` of every identity whose subject is a space member.
    pub issuer: Arc<str>,
    /// Where uploads, posters, proxies, render jobs and rendered files live.
    pub data_dir: Arc<PathBuf>,
    /// Tells open editors a project, an asset or a render changed.
    pub events: Events,
    /// Signs the asset links a sandboxed frame loads without a cookie.
    pub frame_key: FrameKey,
    /// The byte and render caps.
    pub limits: Limits,
    /// `ffmpeg` and `ffprobe`.
    pub tools: Arc<Tools>,
    /// One permit: the asset tasks run `ffprobe` and `ffmpeg` one at a time.
    pub media: Arc<Semaphore>,
    /// The MCP confirmation flow, when `/mcp` is on.
    pub confirmations: Option<Confirmations>,
    /// Mints the person's tokens for other suite apps; `None` without Registre.
    pub exchanger: Option<Exchanger>,
    /// Where the person's assistant lives.
    pub bulle: Bulle,
}
