//! The API reference at `/docs`: one row per route, checked against the router by a test.

use utoipa::openapi::path::{HttpMethod, OperationBuilder, PathItem};
use utoipa::openapi::response::ResponseBuilder;
use utoipa::openapi::{InfoBuilder, OpenApi, OpenApiBuilder, PathsBuilder};

/// One documented route: method, path under `/api` (or the root), tag, summary, success status.
type Row = (&'static str, &'static str, &'static str, &'static str, &'static str);

const ROUTES: &[Row] = &[
    ("GET", "/version", "meta", "The release this binary was built from", "200"),
    ("GET", "/auth/config", "auth", "Whether sign-in is SSO only and OIDC is on", "200"),
    ("GET", "/auth/oidc", "auth", "Starts Registre sign-in (302)", "302"),
    ("GET", "/auth/oidc/callback", "auth", "Registre's redirect back, sets the session cookie (302)", "302"),
    ("POST", "/auth/oidc/exchange", "auth", "Trades a CLI one-time code for a session token", "200"),
    ("POST", "/auth/oidc/device/exchange", "auth", "Trades a facile-cli device token for a session token", "200"),
    ("POST", "/auth/sync-profile", "auth", "Refreshes the profile from Registre", "200"),
    ("POST", "/auth/backchannel-logout", "auth", "Registre's back-channel logout", "200"),
    ("POST", "/auth/logout", "auth", "Ends the session", "200"),
    ("GET", "/auth/me", "auth", "The signed-in person, or 401", "200"),
    ("GET", "/auth/session", "auth", "The signed-in person, or {\"user\": null}, and bulle_url", "200"),
    ("POST", "/registre/sync", "registre", "Registre's directory nudge", "202"),
    ("POST", "/registre/deletions", "registre", "Registre's signed account deletion", "202"),
    ("GET", "/spaces", "spaces", "The caller's spaces and roles", "200"),
    ("GET", "/projects", "projects", "The projects the caller reaches, newest first (?space_id=&trashed=)", "200"),
    ("POST", "/projects", "projects", "Creates a project from a source; 422 lists each invalid line", "201"),
    ("GET", "/projects/{id}", "projects", "One project: metadata, source and rev", "200"),
    ("PATCH", "/projects/{id}", "projects", "Sets the title, the space or the trash", "200"),
    ("DELETE", "/projects/{id}", "projects", "Deletes a trashed project for good", "204"),
    (
        "PUT",
        "/projects/{id}/source",
        "projects",
        "Saves {source, rev}; 409 on a stale rev, 422 on an invalid source",
        "200",
    ),
    ("POST", "/projects/{id}/edits", "projects", "Applies timeline ops {rev, ops}; 422 names the op error", "200"),
    ("GET", "/projects/{id}/timeline", "projects", "The scenes and clips of the saved source, with its rev", "200"),
    ("GET", "/projects/{id}/events", "projects", "Server-sent events: changed {rev}, asset {name, state}", "200"),
    ("GET", "/projects/{id}/frame", "frame", "The compiled HTML, sandboxed (?scene=n, from 0)", "200"),
    ("GET", "/projects/{id}/assets", "assets", "The assets of a project", "200"),
    (
        "POST",
        "/projects/{id}/assets",
        "assets",
        "Uploads an asset (multipart `file`, optional `name`), streamed",
        "201",
    ),
    (
        "GET",
        "/projects/{id}/assets/{name}",
        "assets",
        "Serves an asset with Range (?variant=poster|proxy, ?frame=)",
        "200",
    ),
    ("DELETE", "/projects/{id}/assets/{name}", "assets", "Deletes an asset", "204"),
    ("GET", "/projects/{id}/renders", "renders", "The renders of a project, newest first", "200"),
    ("POST", "/projects/{id}/renders", "renders", "Queues a render {width?, height?, fps?, range?}", "202"),
    ("POST", "/renders", "renders", "Queues a render of a posted bundle (bearer, scope clap.write)", "202"),
    ("GET", "/renders/{id}", "renders", "One render: state, progress and download_url", "200"),
    (
        "DELETE",
        "/renders/{id}",
        "renders",
        "Cancels a waiting or running render (202), removes a finished one (204)",
        "202",
    ),
    ("GET", "/renders/{id}/events", "renders", "Server-sent events: status, until the render is final", "200"),
    ("GET", "/renders/{id}/file", "renders", "The finished MP4, with Range", "200"),
    ("GET", "/assistant", "assistant", "The caller's Bulle assistant name and accent", "200"),
    ("GET", "/mcp/pending", "mcp", "The person's pending agent actions, own browser session only", "200"),
    ("POST", "/mcp/pending/{id}/confirm", "mcp", "Confirms and runs a pending agent action", "200"),
    ("POST", "/mcp/pending/{id}/dismiss", "mcp", "Dismisses a pending agent action", "200"),
];

fn method(name: &str) -> HttpMethod {
    match name {
        "POST" => HttpMethod::Post,
        "PUT" => HttpMethod::Put,
        "PATCH" => HttpMethod::Patch,
        "DELETE" => HttpMethod::Delete,
        _ => HttpMethod::Get,
    }
}

/// The OpenAPI document of every route in [`ROUTES`].
pub fn document() -> OpenApi {
    let paths = ROUTES.iter().fold(PathsBuilder::new(), |paths, (verb, path, tag, summary, status)| {
        let operation = OperationBuilder::new()
            .tag(*tag)
            .summary(Some(*summary))
            .response(*status, ResponseBuilder::new().description(*summary).build());
        paths.path(*path, PathItem::new(method(verb), operation))
    });
    let info = InfoBuilder::new().title("Clap API").version(crate::version::VERSION).build();
    OpenApiBuilder::new().info(info).paths(paths.build()).build()
}

/// The page's title, description and servers.
pub fn config() -> tronc::apiref::Config {
    tronc::apiref::Config {
        title: "Clap API".to_owned(),
        version: crate::version::VERSION.trim_start_matches('v').to_owned(),
        description: "Video projects held as one text file, edited on the web, by agents and by the clap CLI."
            .to_owned(),
        servers: vec!["/api".to_owned()],
        script_url: String::new(),
    }
}
