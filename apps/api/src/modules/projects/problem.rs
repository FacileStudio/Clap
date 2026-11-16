use serde_json::{Value, json};

/// The stable name of a problem, as `errors[].kind` spells it.
pub fn kind(kind: &scene::ErrorKind) -> &'static str {
    use scene::ErrorKind as Kind;
    match kind {
        Kind::Empty => "empty",
        Kind::UnclosedFrontmatter => "unclosed_frontmatter",
        Kind::BadFrontmatterLine => "bad_frontmatter_line",
        Kind::UnknownKey { .. } => "unknown_key",
        Kind::DuplicateKey { .. } => "duplicate_key",
        Kind::ProjectKeyInScene { .. } => "project_key_in_scene",
        Kind::BadValue { .. } => "bad_value",
        Kind::BadStepMarker => "bad_step_marker",
        Kind::StepMarkerOutsideBlock => "step_marker_outside_block",
        Kind::StepOutOfRange { .. } => "step_out_of_range",
        Kind::RemoteUrl { .. } => "remote_url",
        Kind::BadAssetPath { .. } => "bad_asset_path",
        _ => "invalid",
    }
}

/// Every problem of `errors` as `{line, kind, message}`, in line order.
pub fn problems(errors: &scene::Errors) -> Vec<Value> {
    let problem = |error: &scene::Error| json!({"line": error.line, "kind": kind(&error.kind), "message": error.kind.to_string()});
    errors.0.iter().map(problem).collect()
}

/// The `422` body of a timeline operation that does not apply: tronc's `invalid_argument`
/// envelope, plus `timeline_error`, the engine's stable code such as `overlap`.
pub fn timeline_body(error: &clap_timeline::Error) -> Value {
    let mut body = tronc::errors::envelope(tronc::Code::InvalidArgument.as_str(), &error.to_string(), None);
    body["timeline_error"] = json!(error.code());
    body
}
