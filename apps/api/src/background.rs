//! The background work only one process may run: the render mirror and its sweep, the asset
//! sweep, the trash purge and the media resume. During a zero-downtime deploy two API processes
//! share one database and one volume; a Postgres advisory lock picks one. The other waits for it.

use std::time::Duration;

use sqlx::{PgConnection, PgPool};
use tokio_util::sync::CancellationToken;

use crate::state::AppState;

/// The advisory lock key: `clapbg` in ASCII.
pub const KEY: i64 = 0x636c_6170_6267;

/// How often a waiting process tries for the lock, and the holder checks it still has it.
pub const EVERY: Duration = Duration::from_secs(5);

async fn locked(pool: &PgPool) -> Option<PgConnection> {
    let mut connection = pool.acquire().await.ok()?;
    let taken: bool =
        sqlx::query_scalar("SELECT pg_try_advisory_lock($1)").bind(KEY).fetch_one(&mut *connection).await.ok()?;
    taken.then(|| connection.detach())
}

/// Waits for the lock and answers the connection that holds it: dropping that connection gives
/// the lock back. `None` once `token` is cancelled.
pub async fn hold(pool: &PgPool, token: &CancellationToken) -> Option<PgConnection> {
    loop {
        if let Some(connection) = locked(pool).await {
            return Some(connection);
        }
        tokio::select! {
            () = token.cancelled() => return None,
            () = tokio::time::sleep(EVERY) => {}
        }
    }
}

fn start(state: &AppState, token: &CancellationToken) {
    let resumed = state.clone();
    tokio::spawn(async move { crate::modules::assets::worker::resume(&resumed).await });
    tokio::spawn(crate::modules::assets::sweep::run(state.clone(), token.clone()));
    tokio::spawn(crate::modules::renders::mirror::run(state.clone(), token.clone()));
    tokio::spawn(crate::modules::projects::purge::run(state.clone(), token.clone()));
}

async fn kept(connection: &mut PgConnection, token: &CancellationToken) {
    loop {
        tokio::select! {
            () = token.cancelled() => return,
            () = tokio::time::sleep(EVERY) => {}
        }
        if sqlx::query("SELECT 1").execute(&mut *connection).await.is_err() {
            return tracing::warn!("background: lost the lock's connection, stopping until it is held again");
        }
    }
}

/// Runs the background work while this process holds the lock, until `token` is cancelled. When
/// the lock's connection dies, the work stops and the process waits for the lock again.
pub async fn run(state: AppState, token: CancellationToken) {
    while let Some(mut connection) = hold(&state.pool, &token).await {
        tracing::info!("background: holding the lock, starting the mirror, the sweeps and the purge");
        let work = token.child_token();
        start(&state, &work);
        kept(&mut connection, &token).await;
        work.cancel();
    }
}
