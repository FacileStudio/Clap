use serde::{Deserialize, Serialize};
use tronc::{Error, Result};

use super::info::shared;
use super::pages::escape;
use super::poster;
use crate::state::AppState;

/// The width of the embed when the consumer names none.
pub const WIDTH: u32 = 960;

/// `?url=&format=&maxwidth=&maxheight=`.
#[derive(Debug, Default, Deserialize)]
pub struct Query {
    pub url: Option<String>,
    pub format: Option<String>,
    pub maxwidth: Option<u32>,
    pub maxheight: Option<u32>,
}

/// An oEmbed 1.0 `rich` answer.
#[derive(Debug, Serialize)]
pub struct Embed {
    pub version: &'static str,
    #[serde(rename = "type")]
    pub kind: &'static str,
    pub provider_name: &'static str,
    pub provider_url: String,
    pub title: String,
    pub html: String,
    pub width: u32,
    pub height: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumbnail_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumbnail_width: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumbnail_height: Option<i32>,
    /// The length of the video in milliseconds.
    pub duration_ms: i64,
}

/// The token of a share URL of this host, `<public url>/v/<token>`. Anything else names nothing:
/// another host, a private project URL, a path below the page.
pub fn token<'a>(public_url: &str, url: &'a str) -> Option<&'a str> {
    let rest = url.strip_prefix(public_url)?.strip_prefix("/v/")?;
    let token = rest.split(['?', '#']).next()?.trim_end_matches('/');
    let plain = token.bytes().all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_');
    (!token.is_empty() && plain).then_some(token)
}

/// The embed size for a video of `stage`: [`WIDTH`] wide at the video's ratio, shrunk to fit
/// `maxwidth` and `maxheight`.
pub fn size(stage: (u32, u32), maxwidth: Option<u32>, maxheight: Option<u32>) -> (u32, u32) {
    let (stage_width, stage_height) = (u64::from(stage.0.max(1)), u64::from(stage.1.max(1)));
    let mut width = u64::from(maxwidth.unwrap_or(WIDTH).clamp(1, WIDTH));
    let mut height = (width * stage_height / stage_width).max(1);
    if let Some(cap) = maxheight.map(u64::from).filter(|cap| *cap >= 1 && *cap < height) {
        height = cap;
        width = (height * stage_width / stage_height).max(1);
    }
    (width as u32, height as u32)
}

fn iframe(state: &AppState, token: &str, (width, height): (u32, u32), title: &str) -> String {
    format!(
        "<iframe src=\"{}/embed/{token}\" width=\"{width}\" height=\"{height}\" title=\"{}\" frameborder=\"0\" \
         allow=\"autoplay; fullscreen; picture-in-picture\" allowfullscreen></iframe>",
        state.public_url,
        escape(title)
    )
}

/// `GET /oembed`: the card of a live share URL of this host, and `404` for anything else, so a
/// private project never shows that it exists.
pub async fn embed(state: &AppState, query: &Query) -> Result<Embed> {
    let missing = || Error::not_found("not found");
    let token = query.url.as_deref().and_then(|url| token(&state.public_url, url)).ok_or_else(missing)?;
    let shared = shared(state, token).await.map_err(|_| missing())?;
    let stage = (shared.width.max(1) as u32, shared.height.max(1) as u32);
    let (width, height) = size(stage, query.maxwidth, query.maxheight);
    let pictured = poster::exists(state, shared.id).await;
    let thumbnail = poster::size(shared.width, shared.height);
    Ok(Embed {
        version: "1.0",
        kind: "rich",
        provider_name: "Clap",
        provider_url: state.public_url.to_string(),
        html: iframe(state, token, (width, height), &shared.title),
        duration_ms: shared.duration_ms(),
        title: shared.title,
        width,
        height,
        thumbnail_url: pictured.then(|| format!("{}{}", state.public_url, poster::route(token))),
        thumbnail_width: pictured.then_some(thumbnail.0),
        thumbnail_height: pictured.then_some(thumbnail.1),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_a_share_url_of_this_host_names_a_token() {
        let host = "https://clap.test";
        assert_eq!(token(host, "https://clap.test/v/abc-DEF_1"), Some("abc-DEF_1"));
        assert_eq!(token(host, "https://clap.test/v/abc/?x=1#y"), Some("abc"));
        for url in [
            "https://evil.test/v/abc",
            "https://clap.test.evil.test/v/abc",
            "https://clap.test/projects/6f1e2d3c-4b5a-4968-8776-655443322110",
            "https://clap.test/v/",
            "https://clap.test/v/abc/thumbnail.jpg",
            "https://clap.test/embed/abc",
            "https://clap.test/p/abc",
            "/v/abc",
        ] {
            assert_eq!(token(host, url), None, "{url}");
        }
    }

    #[test]
    fn the_embed_keeps_the_video_ratio_within_the_caps() {
        assert_eq!(size((1920, 1080), None, None), (960, 540));
        assert_eq!(size((1024, 768), None, None), (960, 720));
        assert_eq!(size((1920, 1080), Some(480), None), (480, 270));
        assert_eq!(size((1920, 1080), Some(4000), Some(270)), (480, 270));
        assert_eq!(size((1920, 1080), Some(0), Some(0)), (1, 1));
        assert_eq!(size((1080, 1920), None, Some(540)), (303, 540));
    }
}
