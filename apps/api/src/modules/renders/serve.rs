use axum::extract::{Path, Request, State};
use axum::http::{HeaderValue, header};
use axum::response::Response;
use porte::Identity;
use tronc::{Error, Result};

use super::{service, status};
use crate::extract::id;
use crate::modules::projects::access::Access;
use crate::modules::projects::service::fetch;
use crate::state::AppState;

/// The download name of a render: the project title in plain ASCII, or `render`.
pub fn file_name(title: &str) -> String {
    let kept = |c: char| if c.is_ascii_alphanumeric() || c == '-' || c == '_' { c } else { '-' };
    let clean: String = title.trim().chars().map(kept).take(80).collect();
    let clean = clean.trim_matches('-');
    format!("{}.mp4", if clean.is_empty() { "render" } else { clean })
}

/// `GET /renders/{id}/file`: the finished MP4, with range requests. `412` until the render is
/// done.
pub async fn file(
    State(state): State<AppState>,
    who: Identity,
    Path(raw): Path<String>,
    request: Request,
) -> Result<Response> {
    let row = service::authorized(&state, who.user_id, id(&raw)?, Access::Viewer).await?;
    if row.state != "done" {
        return Err(Error::failed(format!("the render is {}, not done: there is no file", row.state)));
    }
    let title = match row.project_id {
        Some(project) => fetch(&state, project).await?.title,
        None => String::new(),
    };
    let mut response = super::output::send(&status::output(&state.data_dir, row.id), request.headers()).await?;
    let headers = response.headers_mut();
    let disposition = format!("attachment; filename=\"{}\"", file_name(&title));
    headers.insert(header::CONTENT_TYPE, HeaderValue::from_static("video/mp4"));
    headers.insert(header::X_CONTENT_TYPE_OPTIONS, HeaderValue::from_static("nosniff"));
    headers.insert(header::CACHE_CONTROL, HeaderValue::from_static("private, max-age=3600"));
    if let Ok(disposition) = HeaderValue::from_str(&disposition) {
        headers.insert(header::CONTENT_DISPOSITION, disposition);
    }
    Ok(response)
}

#[cfg(test)]
mod tests {
    #[test]
    fn a_download_is_named_after_its_project_in_plain_ascii() {
        assert_eq!(super::file_name(" Launch day: été \"2026\" "), "Launch-day---t---2026.mp4");
        assert_eq!(super::file_name("../.."), "render.mp4");
        assert_eq!(super::file_name(""), "render.mp4");
    }
}
