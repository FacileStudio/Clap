//! The scaffold against a real Postgres: probes, the porte schema, sign-in routes and the API reference.

use api::auth::Kits;
use api::env::Config;
use api::state::AppState;
use axum::Router;
use axum::body::Body;
use http::{Request, StatusCode, header};
use http_body_util::BodyExt;
use serde_json::{Value, json};
use sqlx::PgPool;
use tower::ServiceExt;
use tronc::env::Vars;

struct App {
    router: Router,
    kits: Kits,
    pool: PgPool,
}

async fn app(prefix: &str) -> Option<App> {
    let Some(pool) = tronc::testdb::pool(prefix, &api::MIGRATOR).await else {
        eprintln!("{}", tronc::testdb::skip_reason("mise run db-up (postgres:16-alpine on 127.0.0.1:5672)"));
        return None;
    };
    let config = Config::from_vars(&Vars::from_pairs(vec![("DATABASE_URL", "postgres://unused/x")])).unwrap();
    let kits = Kits::new(&config, &pool).await.unwrap();
    let router = api::router::build_router(AppState { pool: pool.clone() }, &kits).unwrap();
    Some(App { router, kits, pool })
}

async fn call(app: &App, method: &str, path: &str, token: Option<&str>) -> (StatusCode, Value) {
    let mut request = Request::builder().method(method).uri(path).header(header::HOST, "clap.test");
    if let Some(token) = token {
        request = request.header(header::AUTHORIZATION, format!("Bearer {token}"));
    }
    let response = app.router.clone().oneshot(request.body(Body::empty()).unwrap()).await.unwrap();
    let status = response.status();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    (status, serde_json::from_slice(&bytes).unwrap_or(Value::Null))
}

#[tokio::test]
async fn probes_version_and_unknown_api_routes_answer() {
    let Some(app) = app("clap_shell").await else { return };
    for path in ["/health", "/ready", "/api/health", "/api/ready", "/version", "/docs"] {
        assert_eq!(call(&app, "GET", path, None).await.0, StatusCode::OK, "{path}");
    }
    assert_eq!(call(&app, "GET", "/version", None).await.1, json!({"version": "dev"}));
    let (status, body) = call(&app, "GET", "/api/nope", None).await;
    assert_eq!((status, &body["error"]["code"]), (StatusCode::NOT_FOUND, &json!("not_found")));
}

#[tokio::test]
async fn a_session_is_anonymous_until_porte_issues_one() {
    let Some(app) = app("clap_session").await else { return };
    assert_eq!(call(&app, "GET", "/api/auth/session", None).await, (StatusCode::OK, json!({"user": null})));
    assert_eq!(call(&app, "GET", "/api/auth/me", None).await.0, StatusCode::UNAUTHORIZED);
    let config = call(&app, "GET", "/api/auth/config", None).await.1;
    assert_eq!(config["sso_only"], json!(true), "{config}");
    let user_id: i64 = sqlx::query_scalar("INSERT INTO porte_users (email, name) VALUES ($1, 'Ada') RETURNING id")
        .bind(format!("ada-{}@example.test", std::process::id()))
        .fetch_one(&app.pool)
        .await
        .unwrap();
    let (token, _) = app.kits.sessions.issue(user_id, "test").await.unwrap();
    let (status, body) = call(&app, "GET", "/api/auth/session", Some(&token)).await;
    assert_eq!((status, &body["user"]["name"]), (StatusCode::OK, &json!("Ada")));
    assert_eq!(call(&app, "GET", "/api/auth/me", Some(&token)).await.0, StatusCode::OK);
    sqlx::query("DELETE FROM porte_users WHERE id = $1").bind(user_id).execute(&app.pool).await.unwrap();
}

#[tokio::test]
async fn every_mounted_route_is_in_the_api_reference() {
    let Some(app) = app("clap_apiref").await else { return };
    let missing = tronc::apiref::undocumented(&app.router, &api::apiref::document(), &api::apiref::config(), &[]);
    assert!(missing.is_empty(), "routes missing from the API document: {missing:?}");
    let (_, document) = call(&app, "GET", "/docs/openapi.json", None).await;
    assert_eq!(document["info"]["title"], json!("Clap API"));
}
