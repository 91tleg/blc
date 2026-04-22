use lambda_http::{Body, Request, Response};

use crate::application::ports::{EventsRepo, RegistrationsRepo};
use crate::application::services::{AuthService, Clock, PosterStorage};
use crate::handlers;
use crate::utils::response::{cors_preflight, HttpResponse};

/// Central dispatcher. Matches (method, path pattern) and delegates to the
/// appropriate handler. All dependency injection happens in main.rs — this
/// function only routes.
pub async fn dispatch(
    req: Request,
    events_repo: &dyn EventsRepo,
    registrations_repo: &dyn RegistrationsRepo,
    clock: &dyn Clock,
    storage: &dyn PosterStorage,
    auth: &dyn AuthService,
) -> HttpResponse {
    let method = req.method().as_str();
    let path = req.uri().path();

    if method == "OPTIONS" {
        return cors_preflight();
    }

    let path = path.trim_end_matches('/');

    match (method, path) {
        ("POST", "/auth/login") => handlers::admin_login::handle(req, auth).await,

        ("GET", "/events") => handlers::list_events::handle(req, events_repo).await,

        ("GET", "/events/poster-upload-url") => {
            handlers::get_poster_upload_url::handle(req, storage, auth).await
        }

        ("GET", p) if p.starts_with("/events/") && !p["/events/".len()..].contains('/') => {
            handlers::get_event::handle(req, events_repo).await
        }

        ("POST", "/events") => {
            handlers::create_event::handle(req, events_repo, clock, storage, auth).await
        }

        ("POST", p) if p.ends_with("/registrations") => {
            handlers::register_for_event::handle(req, events_repo, registrations_repo, clock).await
        }

        ("GET", p) if p.ends_with("/registrations") => {
            handlers::list_registrations::handle(req, events_repo, registrations_repo, auth).await
        }

        _ => not_found(),
    }
}

fn not_found() -> HttpResponse {
    use serde_json::json;
    let body = serde_json::to_string(&json!({
        "error": { "code": "NOT_FOUND", "message": "Route not found" }
    }))
    .unwrap_or_default();
    Response::builder()
        .status(404)
        .header("Content-Type", "application/json")
        .header("Access-Control-Allow-Origin", "*")
        .header("Access-Control-Allow-Headers", "Content-Type, Authorization")
        .header("Access-Control-Allow-Methods", "GET, POST, OPTIONS")
        .body(Body::Text(body))
        .expect("response build failed")
}
