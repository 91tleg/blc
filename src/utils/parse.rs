use lambda_http::{Request, RequestExt};
use serde::de::DeserializeOwned;

use crate::utils::response::HttpResponse;

/// Deserializes the request body as JSON.
/// Returns a 400 response if the body is missing, empty, or not valid JSON.
pub fn json_body<T: DeserializeOwned>(req: &Request) -> Result<T, HttpResponse> {
    let bytes = req.body().as_ref();
    serde_json::from_slice(bytes).map_err(|e| {
        tracing::warn!(error = %e, "failed to parse request body");
        bad_request("Invalid or missing JSON body")
    })
}

/// Reads a path parameter by name, returning an owned String.
/// `path_parameters()` returns a temporary QueryMap so we cannot return a &str into it.
pub fn path_param(req: &Request, name: &str) -> Result<String, HttpResponse> {
    req.path_parameters()
        .first(name)
        .map(str::to_owned)
        .ok_or_else(|| bad_request(format!("missing path parameter: {name}")))
}

/// Reads an optional query string parameter by name, returning an owned String.
pub fn query_param(req: &Request, name: &str) -> Option<String> {
    req.query_string_parameters().first(name).map(str::to_owned)
}

/// Reads a required query string parameter by name.
pub fn required_query_param(req: &Request, name: &str) -> Result<String, HttpResponse> {
    query_param(req, name).ok_or_else(|| bad_request(format!("missing query parameter: {name}")))
}

// ── Internal ──────────────────────────────────────────────────────────────────

fn bad_request(msg: impl Into<String>) -> HttpResponse {
    use lambda_http::{Body, Response};
    use serde_json::json;
    let body = serde_json::to_string(&json!({
        "error": { "code": "BAD_REQUEST", "message": msg.into() }
    }))
    .unwrap_or_default();
    Response::builder()
        .status(400)
        .header("Content-Type", "application/json")
        .body(Body::Text(body))
        .expect("failed to build bad_request response")
}
