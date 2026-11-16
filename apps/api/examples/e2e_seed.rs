//! Test-only seeding for the browser end-to-end run (`apps/client/e2e`). Never shipped: the
//! image builds the `api` and `worker` binaries only. It writes one person straight into a
//! scratch database, with an identity under `OIDC_ISSUER` (empty when unset, as the API reads
//! it), and mints them a browser login session (a `b~` token, as a real sign-in mints) through
//! porte's own session store. It prints the session token as JSON: `{"cookie": "…"}`.
//!
//! `DATABASE_URL=... cargo run -p api --example e2e_seed`

use std::process::ExitCode;

use porte::pg::Store;
use porte::session::{Deps, Manager};
use sqlx::PgPool;

#[tokio::main]
async fn main() -> ExitCode {
    match seed().await {
        Ok(json) => {
            println!("{json}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("e2e_seed: {error}");
            ExitCode::FAILURE
        }
    }
}

async fn seed() -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let pool = PgPool::connect(&std::env::var("DATABASE_URL")?).await?;
    let issuer = std::env::var("OIDC_ISSUER").unwrap_or_default().trim().to_owned();
    let config = porte::Config { issuer: issuer.clone(), ..porte::Config::default() };
    let sessions = Manager::new(&config, Deps { sessions: Store::new(pool.clone()).sessions(), now: None });
    let user: i64 = sqlx::query_scalar("INSERT INTO porte_users (email, name) VALUES ($1, $2) RETURNING id")
        .bind("alice@e2e.test")
        .bind("Alice Martin")
        .fetch_one(&pool)
        .await?;
    sqlx::query("INSERT INTO porte_identities (user_id, provider, subject) VALUES ($1, $2, $3)")
        .bind(user)
        .bind(&issuer)
        .bind("e2e-alice")
        .execute(&pool)
        .await?;
    let (request, _) = http::Request::get("/").body(())?.into_parts();
    let template = porte::Session { user_id: user, ..porte::Session::default() };
    let (token, _, _) = sessions.issue_cookie_for(&request, template).await?;
    Ok(serde_json::json!({ "cookie": token }))
}
