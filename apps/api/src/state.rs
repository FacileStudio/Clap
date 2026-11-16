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
    /// The uploads and posted bundles on their way in, per person and per space.
    pub inflight: crate::modules::assets::inflight::Inflight,
    /// One permit: the asset tasks queue one media job at a time, so the API holds one staged
    /// copy of an upload at once.
    pub media: Arc<Semaphore>,
    /// The MCP confirmation flow, when `/mcp` is on.
    pub confirmations: Option<Confirmations>,
    /// Verifies the bearer of `POST /api/renders`; `None` without Registre.
    pub machine: Option<Arc<porte::mcpauth::Verifier>>,
    /// The origin share links start with, such as `https://clap.facile.studio`.
    pub public_url: Arc<str>,
    /// The origins besides the app that may frame `/embed/{token}`.
    pub embed_ancestors: Arc<[String]>,
    /// The built client, whose `index.html` the public pages are drawn from.
    pub client_dir: Arc<PathBuf>,
    /// Mints the person's tokens for other suite apps; `None` without Registre.
    pub exchanger: Option<Exchanger>,
    /// Where the person's assistant lives.
    pub bulle: Bulle,
}
