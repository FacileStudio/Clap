use axum::response::{IntoResponse, Response};
use http::StatusCode;
use serde_json::{Value, json};
use tronc::mcp::ToolError;

use super::problem::problems;

/// Why a project write was refused. Every door answers the same three cases: REST as a status,
/// MCP as a tool error.
#[derive(Debug)]
pub enum ProjectError {
    /// Any failure the suite envelope already names.
    Api(tronc::Error),
    /// The write named `sent`, and the project has moved on to `rev`: `409`.
    Stale {
        /// The revision the project is at.
        rev: i64,
        /// The revision the write named.
        sent: i64,
    },
    /// The source does not parse: `422`, every problem with its line.
    Invalid(scene::Errors),
}

impl From<tronc::Error> for ProjectError {
    fn from(error: tronc::Error) -> Self {
        ProjectError::Api(error)
    }
}

fn stale(rev: i64, sent: i64) -> String {
    format!("the project is at rev {rev}, not {sent}: read it again, then retry")
}

/// The `409` body: tronc's `already_exists` envelope, plus the `rev` the project is at.
pub fn stale_body(rev: i64, sent: i64) -> Value {
    let mut body = tronc::errors::envelope(tronc::Code::AlreadyExists.as_str(), &stale(rev, sent), None);
    body["rev"] = json!(rev);
    body
}

/// The `422` body: tronc's `invalid_argument` envelope, plus `errors`, one `{line, kind, message}`
/// per problem in line order.
pub fn invalid_body(errors: &scene::Errors) -> Value {
    let mut body = tronc::errors::envelope(tronc::Code::InvalidArgument.as_str(), &errors.to_string(), None);
    body["errors"] = Value::Array(problems(errors));
    body
}

impl IntoResponse for ProjectError {
    fn into_response(self) -> Response {
        match self {
            ProjectError::Api(error) => error.into_response(),
            ProjectError::Stale { rev, sent } => tronc::json::write(StatusCode::CONFLICT, &stale_body(rev, sent)),
            ProjectError::Invalid(errors) => {
                tronc::json::write(StatusCode::UNPROCESSABLE_ENTITY, &invalid_body(&errors))
            }
        }
    }
}

impl From<ProjectError> for ToolError {
    fn from(error: ProjectError) -> Self {
        match error {
            ProjectError::Api(error) => tronc::mcp::failure(error, "clap_list_projects lists project ids."),
            ProjectError::Stale { rev, sent } => {
                ToolError(format!("{}. clap_get_project answers the current source and rev.", stale(rev, sent)))
            }
            ProjectError::Invalid(errors) => ToolError(format!(
                "the source is not a valid project, nothing was saved:\n{errors}\nclap_format_guide explains the format."
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_stale_write_is_a_409_naming_the_current_rev() {
        let response = ProjectError::Stale { rev: 5, sent: 3 }.into_response();
        assert_eq!(response.status(), StatusCode::CONFLICT);
        let body = stale_body(5, 3);
        assert_eq!((body["error"]["code"].clone(), body["rev"].clone()), (json!("already_exists"), json!(5)));
    }

    #[test]
    fn an_invalid_source_is_a_422_listing_each_line() {
        let errors = scene::parse("# A\n\n---\ncolour: red\n---\n\n# B\n").unwrap_err();
        let body = invalid_body(&errors);
        assert_eq!(body["errors"][0], json!({"line": 4, "kind": "unknown_key", "message": "unknown key `colour`"}));
        assert_eq!(body["error"]["message"], json!("line 4: unknown key `colour`"));
        assert_eq!(ProjectError::Invalid(errors).into_response().status(), StatusCode::UNPROCESSABLE_ENTITY);
    }
}
