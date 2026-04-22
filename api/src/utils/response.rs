use lambda_http::{Body, Response};
use serde::Serialize;
use serde_json::json;

use crate::application::errors::AppError;

pub type HttpResponse = Response<Body>;

pub fn ok(body: impl Serialize) -> HttpResponse {
    json_response(200, body)
}

pub fn created(body: impl Serialize) -> HttpResponse {
    json_response(201, body)
}

pub fn cors_preflight() -> HttpResponse {
    Response::builder()
        .status(204)
        .header("Access-Control-Allow-Origin", "*")
        .header(
            "Access-Control-Allow-Headers",
            "Content-Type, Authorization",
        )
        .header("Access-Control-Allow-Methods", "GET, POST, OPTIONS")
        .body(Body::Empty)
        .expect("failed to build response")
}

fn json_response(status: u16, body: impl Serialize) -> HttpResponse {
    let json = serde_json::to_string(&body).unwrap_or_else(|_| "{}".into());
    Response::builder()
        .status(status)
        .header("Content-Type", "application/json")
        .header("Access-Control-Allow-Origin", "*")
        .header(
            "Access-Control-Allow-Headers",
            "Content-Type, Authorization",
        )
        .header("Access-Control-Allow-Methods", "GET, POST, OPTIONS")
        .body(Body::Text(json))
        .expect("failed to build response")
}

pub fn error_response(err: AppError) -> HttpResponse {
    let (status, code) = match &err {
        AppError::InvalidEvent(_) | AppError::InvalidRegistration(_) => (422, "VALIDATION_ERROR"),
        AppError::EventFull => (400, "EVENT_FULL"),
        AppError::AlreadyRegistered => (400, "ALREADY_REGISTERED"),
        AppError::EventNotFound => (404, "EVENT_NOT_FOUND"),
        AppError::RegistrationNotFound => (404, "REGISTRATION_NOT_FOUND"),
        AppError::AuthError(_) => (401, "INVALID_CREDENTIALS"),
        AppError::StorageError(_) => {
            tracing::error!(error = %err, "storage error");
            (500, "INTERNAL_ERROR")
        }
    };

    json_response(
        status,
        json!({
            "error": {
                "code": code,
                "message": err.to_string()
            }
        }),
    )
}

/// Extracts the Bearer token from the Authorization header.
/// Returns `None` if the header is absent or malformed.
pub fn extract_bearer_token(req: &lambda_http::Request) -> Option<&str> {
    req.headers()
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
}
