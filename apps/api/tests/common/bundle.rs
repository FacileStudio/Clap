use axum::body::Body;
use http::{Request, header};
use serde_json::{Value, json};

use super::upload::{BOUNDARY, PNG};
use super::{Answer, App};

pub const PAGE: &[u8] = b"<!doctype html><img src=\"assets/a.png\"><script>window.__scene = {};</script>";

pub fn body(spec: Option<Value>, files: &[(&str, &[u8])]) -> Vec<u8> {
    let mut body = Vec::new();
    if let Some(spec) = spec {
        let part = format!("--{BOUNDARY}\r\nContent-Disposition: form-data; name=\"spec\"\r\n\r\n{spec}\r\n");
        body.extend_from_slice(part.as_bytes());
    }
    for (path, bytes) in files {
        let head =
            format!("--{BOUNDARY}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"{path}\"\r\n\r\n");
        body.extend_from_slice(head.as_bytes());
        body.extend_from_slice(bytes);
        body.extend_from_slice(b"\r\n");
    }
    body.extend_from_slice(format!("--{BOUNDARY}--\r\n").as_bytes());
    body
}

pub async fn post(app: &App, bearer: Option<&str>, body: Vec<u8>) -> Answer {
    let mut request = Request::post("/api/renders")
        .header(header::HOST, "clap.test")
        .header(header::CONTENT_TYPE, format!("multipart/form-data; boundary={BOUNDARY}"));
    if let Some(bearer) = bearer {
        request = request.header(header::AUTHORIZATION, format!("Bearer {bearer}"));
    }
    super::http::raw(&app.router, request.body(Body::from(body)).unwrap()).await
}

pub fn whole() -> Vec<u8> {
    body(Some(json!({"width": 1280, "height": 720, "fps": 24})), &[("index.html", PAGE), ("assets/a.png", PNG)])
}
