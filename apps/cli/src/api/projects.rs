//! Projects: list, create, read, and write the source against a revision.

use serde::{Deserialize, Serialize};
use serde_json::json;

use super::{ApiError, Client, Payload, text};
use crate::url::encode;

/// A project as the list and the create call answer it.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct Summary {
    /// The project id.
    #[serde(deserialize_with = "text")]
    pub id: String,
    /// The project title.
    #[serde(default)]
    pub title: String,
    /// The current revision, when the server lists it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rev: Option<i64>,
    /// When the project last changed, when the server lists it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,
}

/// A project with its source.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Record {
    /// The project id.
    #[serde(deserialize_with = "text")]
    pub id: String,
    /// The project title.
    #[serde(default)]
    pub title: String,
    /// The whole source text.
    pub source: String,
    /// The revision this source is.
    pub rev: i64,
}

#[derive(Deserialize)]
struct Written {
    rev: i64,
}

impl Client {
    /// `GET /api/projects`.
    pub fn projects(&self) -> Result<Vec<Summary>, ApiError> {
        self.list("/projects")
    }

    /// `POST /api/projects`: a new personal project holding `source`, at revision 1.
    pub fn create_project(&self, title: &str, source: &str) -> Result<Summary, ApiError> {
        self.json("POST", "/projects", Payload::Json(&json!({ "title": title, "source": source })))
    }

    /// `GET /api/projects/{id}`.
    pub fn project(&self, id: &str) -> Result<Record, ApiError> {
        self.json("GET", &format!("/projects/{}", encode(id)), Payload::Empty)
    }

    /// `PUT /api/projects/{id}/source`: writes `source` over revision `rev` and answers the new
    /// revision. A stale `rev` is refused with `409`, a source that does not parse with `422`.
    pub fn put_source(&self, id: &str, source: &str, rev: i64) -> Result<i64, ApiError> {
        let body = json!({ "source": source, "rev": rev });
        let written: Written = self.json("PUT", &format!("/projects/{}/source", encode(id)), Payload::Json(&body))?;
        Ok(written.rev)
    }
}
