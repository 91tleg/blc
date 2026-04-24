use lambda_http::{Body, Request, Response};

use crate::application::ports::{EventsRepo, RegistrationsRepo};
use crate::application::services::{AuthService, Clock, RegistrationEmailSender};
use crate::handlers;
use crate::infrastructure::s3::poster_storage::S3PosterStorage;
use crate::utils::response::{cors_preflight, HttpResponse};

/// Central dispatcher. Matches (method, path pattern) and delegates to the
/// appropriate handler. All dependency injection happens in main.rs — this
/// function only routes.
pub async fn dispatch(
    req: Request,
    events_repo: &dyn EventsRepo,
    registrations_repo: &dyn RegistrationsRepo,
    email_sender: &dyn RegistrationEmailSender,
    clock: &dyn Clock,
    storage: &S3PosterStorage,
    auth: &dyn AuthService,
) -> HttpResponse {
    let method = req.method().as_str();
    let path = req.uri().path();

    if method == "OPTIONS" {
        return cors_preflight();
    }

    let path = path.trim_end_matches('/');
    let path = path
        .strip_prefix("/prod")
        .filter(|suffix| suffix.is_empty() || suffix.starts_with('/'))
        .unwrap_or(path);
    let path = if path.is_empty() { "/" } else { path };

    match (method, path) {
        ("POST", "/auth/login") => handlers::admin_login::handle(req, auth).await,

        ("GET", "/events") => handlers::list_events::handle(req, events_repo).await,

        ("GET", "/events/poster-upload-url") => {
            handlers::get_poster_upload_url::handle(req, storage, auth).await
        }

        ("GET", p) if p.ends_with("/posters") => {
            handlers::event_posters::list(req, events_repo).await
        }

        ("POST", p) if p.ends_with("/posters") => {
            handlers::event_posters::save(req, events_repo, storage).await
        }

        ("DELETE", p) if p.contains("/posters/") => {
            handlers::event_posters::delete(req, events_repo, storage).await
        }

        ("GET", p) if p.starts_with("/events/") && !p["/events/".len()..].contains('/') => {
            handlers::get_event::handle(req, events_repo).await
        }

        ("POST", "/events") => {
            handlers::create_event::handle(req, events_repo, clock, storage, auth).await
        }

        ("POST", p) if p.ends_with("/registrations") => {
            handlers::register_for_event::handle(
                req,
                events_repo,
                registrations_repo,
                email_sender,
                clock,
            )
            .await
        }

        ("GET", p) if p.ends_with("/registrations") => {
            handlers::list_registrations::handle(req, events_repo, registrations_repo).await
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
        .header(
            "Access-Control-Allow-Headers",
            "Content-Type, Authorization",
        )
        .header("Access-Control-Allow-Methods", "GET, POST, DELETE, OPTIONS")
        .body(Body::Text(body))
        .expect("response build failed")
}
