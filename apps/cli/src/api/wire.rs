//! One request and its answer: the agent, the suite error envelope, and lenient reading.

use std::time::Duration;

use serde::de::DeserializeOwned;
use serde::{Deserialize, Deserializer};
use serde_json::Value;
use ureq::http::{Request, Response};
use ureq::{AsSendBody, Body};

use super::{ApiError, SourceError};

const TIMEOUT: Duration = Duration::from_secs(120);
const BODY_LIMIT: u64 = 64 << 20;

/// The HTTP agent: a timeout, and error statuses answered instead of raised.
pub(crate) fn agent() -> ureq::Agent {
    ureq::Agent::config_builder()
        .timeout_global(Some(TIMEOUT))
        .http_status_as_error(false)
        .user_agent(concat!("clap/", env!("CARGO_PKG_VERSION")))
        .build()
        .into()
}

fn refusal(status: u16, body: &[u8]) -> ApiError {
    #[derive(Deserialize)]
    struct Envelope {
        error: Detail,
        #[serde(default)]
        errors: Vec<SourceError>,
    }
    #[derive(Deserialize)]
    struct Detail {
        #[serde(default)]
        code: String,
        #[serde(default)]
        message: String,
    }
    match serde_json::from_slice::<Envelope>(body) {
        Ok(Envelope { errors, .. }) if status == 422 && !errors.is_empty() => ApiError::Invalid(errors),
        Ok(Envelope { error, .. }) if !error.code.is_empty() => {
            ApiError::Refused { status, code: error.code, message: error.message }
        }
        _ => {
            let text: String = String::from_utf8_lossy(body).chars().take(200).collect();
            ApiError::Refused { status, code: format!("http_{status}"), message: text.trim().to_owned() }
        }
    }
}

/// An agent for a transfer that may last as long as the file is large: only connecting is timed.
pub(crate) fn patient() -> ureq::Agent {
    ureq::Agent::config_builder()
        .timeout_connect(Some(Duration::from_secs(30)))
        .http_status_as_error(false)
        .user_agent(concat!("clap/", env!("CARGO_PKG_VERSION")))
        .build()
        .into()
}

fn limited(response: &mut Response<Body>) -> Result<Vec<u8>, ApiError> {
    let body = response.body_mut().with_config().limit(BODY_LIMIT).read_to_vec();
    body.map_err(|error| ApiError::Unreachable(error.to_string()))
}

/// Sends `request` and answers the response of a success, its body unread, or the error.
pub(crate) fn open(agent: &ureq::Agent, request: Request<impl AsSendBody>) -> Result<Response<Body>, ApiError> {
    let mut response = agent.run(request).map_err(|error| ApiError::Unreachable(error.to_string()))?;
    let status = response.status().as_u16();
    if (200..300).contains(&status) {
        return Ok(response);
    }
    Err(refusal(status, &limited(&mut response)?))
}

/// Sends `request` and answers the body of a success, or the error.
pub(crate) fn run(agent: &ureq::Agent, request: Request<impl AsSendBody>) -> Result<Vec<u8>, ApiError> {
    limited(&mut open(agent, request)?)
}

/// Reads a JSON answer.
pub(crate) fn parse<T: DeserializeOwned>(body: &[u8]) -> Result<T, ApiError> {
    serde_json::from_slice(body).map_err(|error| ApiError::Malformed(error.to_string()))
}

/// Reads an id the server may send as a string or as a number.
pub(crate) fn text<'de, D: Deserializer<'de>>(deserializer: D) -> Result<String, D::Error> {
    Ok(match Value::deserialize(deserializer)? {
        Value::String(text) => text,
        other => other.to_string(),
    })
}
