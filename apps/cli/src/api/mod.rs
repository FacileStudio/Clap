//! Every HTTP call the CLI makes: the Clap API under `/api`, and the identity provider's
//! device grant. A mismatch with the server is fixed in this module and nowhere else.

mod assets;
mod auth;
pub mod device;
mod projects;
mod renders;
mod transfer;
mod wire;

use serde::de::DeserializeOwned;
use serde_json::Value;
use ureq::http::Request;

pub use assets::{RemoteAsset, Upload};
pub use auth::{AuthConfig, Me};
pub use projects::{Record, Summary};
pub use renders::Render;
pub use transfer::OnBytes;
pub(crate) use wire::{agent, open, parse, patient, run, text};

/// Why a call did not give its answer.
#[derive(Debug, Clone, PartialEq)]
pub enum ApiError {
    /// No HTTP answer came back.
    Unreachable(String),
    /// A local file the call was to send could not be read.
    Unreadable(String),
    /// The server answered with an error status, in the suite envelope when it sent one.
    Refused {
        /// The HTTP status.
        status: u16,
        /// `error.code` of the envelope, or `http_<status>` without one.
        code: String,
        /// `error.message` of the envelope, or the start of the body.
        message: String,
    },
    /// `422`: the server parsed the source and refused it, with the line of each problem.
    Invalid(Vec<SourceError>),
    /// A success answer the CLI cannot read.
    Malformed(String),
}

impl ApiError {
    /// Whether this is the `409 conflict` a stale `rev` gets.
    pub fn is_conflict(&self) -> bool {
        matches!(self, ApiError::Refused { status: 409, .. })
    }
}

/// One problem the server found in a source.
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
pub struct SourceError {
    /// The 1-based line.
    pub line: usize,
    /// The problem in words, as `docs/format.md` lists it.
    pub message: String,
}

/// What a request carries.
pub(crate) enum Payload<'a> {
    /// No body.
    Empty,
    /// A JSON body.
    Json(&'a Value),
}

/// A Clap server and the token to call it with.
pub struct Client {
    agent: ureq::Agent,
    server: String,
    token: Option<String>,
}

impl Client {
    /// A client for the server at `server`, a site root such as `https://clap.facile.studio`.
    pub fn new(server: &str, token: Option<&str>) -> Client {
        Client { agent: agent(), server: server.trim_end_matches('/').to_owned(), token: token.map(str::to_owned) }
    }

    /// The site root this client calls.
    pub fn server(&self) -> &str {
        &self.server
    }

    /// A request to `path` of the API, with the token when there is one.
    pub(crate) fn request(&self, method: &str, path: &str) -> ureq::http::request::Builder {
        let request = Request::builder()
            .method(method)
            .uri(format!("{}/api{path}", self.server))
            .header("Accept", "application/json");
        match &self.token {
            Some(token) => request.header("Authorization", format!("Bearer {token}")),
            None => request,
        }
    }

    pub(crate) fn call(&self, method: &str, path: &str, payload: Payload<'_>) -> Result<Vec<u8>, ApiError> {
        let mut request = self.request(method, path);
        let (kind, body) = match payload {
            Payload::Empty => (None, Vec::new()),
            Payload::Json(value) => (Some("application/json"), value.to_string().into_bytes()),
        };
        if let Some(kind) = kind {
            request = request.header("Content-Type", kind);
        }
        let request = request.body(body).map_err(|error| ApiError::Unreachable(error.to_string()))?;
        run(&self.agent, request)
    }

    pub(crate) fn json<T: DeserializeOwned>(&self, method: &str, path: &str, body: Payload<'_>) -> Result<T, ApiError> {
        parse(&self.call(method, path, body)?)
    }

    /// Reads a list the server may send bare or wrapped in an object.
    pub(crate) fn list<T: DeserializeOwned>(&self, path: &str) -> Result<Vec<T>, ApiError> {
        let items = match self.json::<Value>("GET", path, Payload::Empty)? {
            Value::Object(fields) => {
                fields.into_iter().map(|(_, value)| value).find(Value::is_array).unwrap_or_default()
            }
            other => other,
        };
        serde_json::from_value(items).map_err(|error| ApiError::Malformed(error.to_string()))
    }
}
