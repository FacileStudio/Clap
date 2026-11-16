//! The share page of a real video: its poster is made by ffmpeg once the render is kept, and
//! only the embed page may be framed by another site. The poster needs `ffmpeg`.

mod common;

use common::HOST;
use common::media::{clip, installed};
use common::share::{done, link};
use http::header;
use serde_json::json;

#[tokio::test]
async fn a_finished_render_gets_a_poster_that_turns_on_og_image_and_the_oembed_picture() {
    let Some(pool) = common::pool("clap_public_poster").await else { return };
    if !installed() {
        return;
    }
    let app = common::app(pool).await;
    let ada = app.person("ada").await;
    let id = common::render::project(&app, &ada, None).await;
    done(&app, &ada, &id, &clip(&app.state.data_dir)).await;
    let token = link(&app, &ada, &id).await;
    let picture = format!("/v/{token}/thumbnail.jpg");
    let served = app.call("GET", &picture, None, None).await;
    assert_eq!((served.status.as_u16(), &served.bytes[..3]), (200, &[0xff, 0xd8, 0xff][..]));
    assert_eq!(served.headers["content-type"], "image/jpeg");
    let facts = app.call("GET", &format!("/api/public/renders/{token}"), None, None).await.json();
    assert_eq!(facts["poster"], json!(picture));
    let page = app.call("GET", &format!("/v/{token}"), None, None).await.text();
    assert!(page.contains(&format!("<meta property=\"og:image\" content=\"{HOST}{picture}\">")), "{page}");
    let card = app.call("GET", &format!("/oembed?url={HOST}/v/{token}&maxwidth=480"), None, None).await.json();
    assert_eq!((card["thumbnail_url"].clone(), card["width"].clone()), (json!(format!("{HOST}{picture}")), json!(480)));
    assert_eq!((card["thumbnail_width"].clone(), card["thumbnail_height"].clone()), (json!(320), json!(240)));
}

#[tokio::test]
async fn only_the_embed_page_may_be_framed_by_prisme() {
    let Some(pool) = common::pool("clap_public_embed").await else { return };
    let app = common::app(pool).await;
    let ada = app.person("ada").await;
    let (_, token) = common::share::shared(&app, &ada, None).await;
    let security = api::security::config(Default::default(), &app.state.client_dir, "https://bulle.test");
    let layers = tronc::http::layers(tronc::http::Config { security, ..Default::default() }).unwrap();
    let router = app.router.clone().layer(layers);
    let get = |path: String| common::send(&router, "GET", Box::leak(path.into_boxed_str()), None, None);
    let framing = |answer: &common::Answer| {
        let policy = answer.headers["content-security-policy"].to_str().unwrap().to_owned();
        policy.split(';').map(str::trim).find(|part| part.starts_with("frame-ancestors")).unwrap().to_owned()
    };
    let embed = get(format!("/embed/{token}")).await;
    assert_eq!(embed.status, 200);
    assert_eq!(framing(&embed), "frame-ancestors 'self' https://prisme.facile.studio");
    assert!(embed.headers.get("x-frame-options").is_none() && embed.headers.get(header::SET_COOKIE).is_none());
    let page = get(format!("/v/{token}")).await;
    assert_eq!(
        (framing(&page).as_str(), page.headers["x-frame-options"].to_str().unwrap()),
        ("frame-ancestors 'none'", "DENY")
    );
}
