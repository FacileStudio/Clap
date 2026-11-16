//! What a share link opens: no session, no cookie, no source, no ids, and 404 for anything else.

mod common;

use axum::body::Body;
use common::HOST;
use common::share::{VIDEO, shared};
use http::{Request, header};
use serde_json::json;

fn routes(token: &str) -> Vec<String> {
    vec![
        format!("/api/public/renders/{token}"),
        format!("/api/public/renders/{token}/video.mp4"),
        format!("/v/{token}"),
        format!("/embed/{token}"),
        format!("/oembed?url={HOST}/v/{token}&format=json"),
    ]
}

#[tokio::test]
async fn a_shared_render_answers_its_facts_and_video_without_source_ids_or_cookies() {
    let Some(pool) = common::pool("clap_public").await else { return };
    let app = common::app(pool).await;
    let ada = app.person("ada").await;
    let (id, token) = shared(&app, &ada, None).await;
    let login = common::http::browser_login(&app.kits.sessions, ada.user_id).await;
    for path in routes(&token) {
        let answer = common::http::with_cookie(&app.router, "GET", &path, &login).await;
        assert_eq!(answer.status, 200, "{path}: {}", answer.text());
        assert!(answer.headers.get(header::SET_COOKIE).is_none(), "{path}");
        let body = answer.text();
        assert!(!body.contains(&id) && !body.contains("data-start") && !body.contains("assets/dot.png"), "{path}");
    }
    let facts = app.call("GET", &format!("/api/public/renders/{token}"), None, None).await.json();
    let video = format!("/api/public/renders/{token}/video.mp4");
    let expected = json!({"title": "Launch \"day\" <b>", "duration_ms": 2000, "width": 320, "height": 240,
        "poster": null, "video": video});
    assert_eq!(facts, expected);
    let request = Request::get(&video).header(header::RANGE, "bytes=4-11").body(Body::empty()).unwrap();
    let part = common::http::raw(&app.router, request).await;
    assert_eq!((part.status.as_u16(), part.bytes.as_slice()), (206, &VIDEO[4..12]));
    assert_eq!(part.headers["content-type"], "video/mp4");
    assert_eq!(part.headers["access-control-allow-origin"], "*");
    assert!(part.headers.get(header::SET_COOKIE).is_none());
}

#[tokio::test]
async fn an_unshared_unknown_or_revoked_link_is_not_found_everywhere() {
    let Some(pool) = common::pool("clap_public_404").await else { return };
    let app = common::app(pool).await;
    let ada = app.person("ada").await;
    let private = common::render::project(&app, &ada, None).await;
    let unknown = porte::new_token().unwrap();
    for token in [unknown.as_str(), private.as_str(), "..", "a"] {
        for path in routes(token).into_iter().chain([format!("/v/{token}/thumbnail.jpg")]) {
            let answer = app.call("GET", &path, None, None).await;
            assert_eq!(answer.status, 404, "{path}");
            assert!(!answer.text().contains("Launch"), "{path}: {}", answer.text());
        }
    }
    let (id, token) = shared(&app, &ada, None).await;
    for url in [
        format!("{HOST}/projects/{id}"),
        format!("{HOST}/projects/{private}"),
        format!("https://evil.test/v/{token}"),
        format!("{HOST}/embed/{token}"),
        format!("{HOST}/v/{token}/thumbnail.jpg"),
    ] {
        assert_eq!(app.call("GET", &format!("/oembed?url={url}"), None, None).await.status, 404, "{url}");
    }
    assert_eq!(app.call("GET", "/oembed", None, None).await.status, 404);
    assert_eq!(app.call("GET", &format!("/oembed?url={HOST}/v/{token}&format=xml"), None, None).await.status, 501);
    assert_eq!(app.call("DELETE", &format!("/api/projects/{id}/share"), Some(&ada), None).await.status, 204);
    for path in routes(&token) {
        assert_eq!(app.call("GET", &path, None, None).await.status, 404, "revoked {path}");
    }
}

#[tokio::test]
async fn the_page_and_oembed_describe_the_video_without_a_picture_while_none_exists() {
    let Some(pool) = common::pool("clap_public_card").await else { return };
    let app = common::app(pool).await;
    let ada = app.person("ada").await;
    let (_, token) = shared(&app, &ada, None).await;
    let page = app.call("GET", &format!("/v/{token}"), None, None).await.text();
    assert!(page.contains("<meta name=\"robots\" content=\"noindex\">"), "{page}");
    assert!(page.contains("<meta property=\"og:title\" content=\"Launch &quot;day&quot; &lt;b&gt;\">"), "{page}");
    assert!(page.contains(&format!("<meta property=\"og:url\" content=\"{HOST}/v/{token}\">")), "{page}");
    let video = format!("<meta property=\"og:video\" content=\"{HOST}/api/public/renders/{token}/video.mp4\">");
    assert!(page.contains(&video) && page.contains("og:video:type"), "{page}");
    assert!(page.contains("application/json+oembed") && !page.contains("og:image"), "{page}");
    assert_eq!(app.call("GET", &format!("/v/{token}/thumbnail.jpg"), None, None).await.status, 404);
    let card = app.call("GET", &format!("/oembed?url={HOST}/v/{token}&format=json"), None, None).await.json();
    let iframe = format!(
        "<iframe src=\"{HOST}/embed/{token}\" width=\"960\" height=\"720\" title=\"Launch &quot;day&quot; &lt;b&gt;\" \
         frameborder=\"0\" allow=\"autoplay; fullscreen; picture-in-picture\" allowfullscreen></iframe>"
    );
    let expected = json!({"version": "1.0", "type": "rich", "provider_name": "Clap", "provider_url": HOST,
        "title": "Launch \"day\" <b>", "html": iframe, "width": 960, "height": 720, "duration_ms": 2000});
    assert_eq!(card, expected);
}
