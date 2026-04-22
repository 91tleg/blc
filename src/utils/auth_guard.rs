use lambda_http::{Body, Request, Response};
use serde_json::json;

use crate::application::services::AuthService;
use crate::utils::response::HttpResponse;

/// Extracts and verifies the admin Bearer token.
/// Returns `Err(403 response)` if the token is absent or invalid.
pub fn require_admin(req: &Request, auth: &dyn AuthService) -> Result<(), HttpResponse> {
    let token = extract_bearer_token(req).ok_or_else(forbidden)?;
    auth.verify_admin_token(token).map_err(|_| forbidden())
}

fn extract_bearer_token(req: &Request) -> Option<&str> {
    req.headers()
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
}

fn forbidden() -> HttpResponse {
    let body = serde_json::to_string(&json!({
        "error": { "code": "FORBIDDEN", "message": "Admin token required" }
    }))
    .unwrap_or_default();
    Response::builder()
        .status(403)
        .header("Content-Type", "application/json")
        .body(Body::Text(body))
        .expect("response build failed")
}
