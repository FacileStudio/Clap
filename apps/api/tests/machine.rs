//! `POST /api/renders`: a bundle posted by another suite app with a Registre token it exchanged
//! for the person. The tokens here are signed by a test key the fake issuer publishes.

mod common;

use common::bundle::{PAGE, body, post, whole};
use common::idp::{self, token};
use common::upload::PNG;
use common::{Answer, App};
use serde_json::{Value, json};

async fn get(app: &App, bearer: &str, path: &str) -> Answer {
    common::send(&app.router, "GET", path, Some(bearer), None).await
}

async fn owner(app: &App, render: &Value) -> String {
    let query =
        "SELECT i.subject FROM renders r JOIN porte_identities i ON i.user_id = r.requested_by WHERE r.id = $1::uuid";
    sqlx::query_scalar(query).bind(render["id"].as_str().unwrap()).fetch_one(&app.pool).await.unwrap()
}

#[tokio::test]
async fn an_exchanged_token_queues_a_bundle_for_its_person_and_creates_the_account() {
    let Some(pool) = common::pool("clap_machine").await else { return };
    let app = common::app_with(pool, &idp::REGISTRE).await;
    let subject = format!("zoe-{}", uuid::Uuid::new_v4());
    let queued = post(&app, Some(&token(&subject, json!({}))), whole()).await;
    assert_eq!(queued.status, 202, "{}", queued.text());
    let render = queued.json();
    assert_eq!(
        (render["state"].clone(), render["project_id"].clone(), render["fps"].clone()),
        (json!("queued"), Value::Null, json!(24))
    );
    let owner = owner(&app, &render).await;
    assert_eq!(owner, subject);
    let dir = common::render::dir(&app, &render);
    assert_eq!(std::fs::read(dir.join("bundle/index.html")).unwrap(), PAGE);
    assert_eq!(std::fs::read(dir.join("bundle/assets/a.png")).unwrap(), PNG);
    let job: Value = serde_json::from_slice(&std::fs::read(dir.join("job.json")).unwrap()).unwrap();
    assert_eq!(
        (job["id"].clone(), job["width"].clone(), job["range"].clone()),
        (render["id"].clone(), json!(1280), Value::Null)
    );
    let one = format!("/api/renders/{}", render["id"].as_str().unwrap());
    let reader = token(&subject, json!({"scope": "clap.read"}));
    assert_eq!(get(&app, &reader, &one).await.json()["id"], render["id"]);
    let stranger = token(&format!("yan-{}", uuid::Uuid::new_v4()), json!({}));
    assert_eq!(post(&app, Some(&stranger), whole()).await.status, 202);
    assert_eq!(get(&app, &stranger, &one).await.status, 404);
}

#[tokio::test]
async fn a_token_of_the_wrong_audience_scope_or_shape_is_refused_and_queues_nothing() {
    let Some(pool) = common::pool("clap_machine_refused").await else { return };
    let app = common::app_with(pool, &idp::REGISTRE).await;
    let subject = format!("ada-{}", uuid::Uuid::new_v4());
    let refused = [
        (token(&subject, json!({"aud": "diapo"})), 401),
        (token(&subject, json!({"act": null})), 401),
        (token(&subject, json!({"exp": 1000})), 401),
        (token(&subject, json!({"iss": "https://evil.test"})), 401),
        (format!("{}x", token(&subject, json!({}))), 401),
        ("not-a-jwt".to_owned(), 401),
        (token(&subject, json!({"scope": "clap.read"})), 403),
        (token(&subject, json!({"scope": "diapo.write"})), 403),
    ];
    for (bearer, status) in refused {
        assert_eq!(post(&app, Some(&bearer), whole()).await.status, status, "{bearer}");
    }
    assert_eq!(post(&app, None, whole()).await.status, 401);
    let session = app.person("eve").await;
    assert_eq!(post(&app, Some(&session.token), whole()).await.status, 401);
    assert_eq!(common::render::jobs(&app), 0);
}

#[tokio::test]
async fn a_bundle_is_held_to_safe_paths_an_index_a_spec_and_the_caps() {
    let Some(pool) = common::pool("clap_machine_bundle").await else { return };
    let vars = [idp::REGISTRE.as_slice(), &[("RENDER_BUNDLE_MAX_BYTES", "256")]].concat();
    let app = common::app_with(pool, &vars).await;
    let bearer = token(&format!("ada-{}", uuid::Uuid::new_v4()), json!({}));
    let spec = || Some(json!({"width": 1280, "height": 720}));
    let refused = [
        (body(spec(), &[("index.html", PAGE), ("../escape.png", PNG)]), 400),
        (body(spec(), &[("index.html", PAGE), ("/etc/passwd", PNG)]), 400),
        (body(spec(), &[("assets/a.png", PNG)]), 400),
        (body(None, &[("index.html", PAGE)]), 400),
        (body(Some(json!({"height": 720})), &[("index.html", PAGE)]), 400),
        (body(Some(json!({"width": 4000, "height": 2000})), &[("index.html", PAGE)]), 400),
        (body(spec(), &[("index.html", PAGE), ("big.bin", &[0u8; 300])]), 413),
    ];
    for (bundle, status) in refused {
        let answer = post(&app, Some(&bearer), bundle).await;
        assert_eq!(answer.status, status, "{}", answer.text());
    }
    assert_eq!(common::render::jobs(&app), 0);
    assert_eq!(std::fs::read_dir(app.state.data_dir.join("uploads")).unwrap().count(), 0);
    common::render::heartbeat(&app.state.data_dir, time::OffsetDateTime::UNIX_EPOCH);
    assert_eq!(post(&app, Some(&bearer), body(spec(), &[("index.html", PAGE)])).await.status, 503);
}

#[tokio::test]
async fn a_token_exactly_as_registre_exchanges_it_queues_a_render_for_a_new_person() {
    let Some(pool) = common::pool("clap_machine_exchanged").await else { return };
    let app = common::app_with(pool, &idp::REGISTRE).await;
    let subject = format!("first-{}", uuid::Uuid::new_v4());
    let queued = post(&app, Some(&idp::exchanged(&subject, "diapo")), whole()).await;
    assert_eq!(queued.status, 202, "{}", queued.text());
    assert_eq!(owner(&app, &queued.json()).await, subject, "the person's account was created on this first call");
    let again = post(&app, Some(&idp::exchanged(&subject, "diapo")), whole()).await;
    assert_eq!(again.status, 202, "{}", again.text());
    let unverified = idp::token(&format!("lie-{}", uuid::Uuid::new_v4()), json!({"email_verified": false}));
    assert_eq!(post(&app, Some(&unverified), whole()).await.status, 401, "an explicit false is never trusted");
}
