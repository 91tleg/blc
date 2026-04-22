use chrono::DateTime;
use lambda_http::Request;
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

use crate::application::{
    create_event::{create_event, CreateEventInput},
    errors::AppError,
    ports::EventsRepo,
    services::{AuthService, Clock, PosterStorage},
};
use crate::domain::{event::EventError, types::EventId};
use crate::utils::{
    auth_guard::require_admin,
    parse::json_body,
    response::{created, error_response, HttpResponse},
};

#[derive(Deserialize)]
struct Body {
    name: String,
    description: Option<String>,
    location: String,
    starts_at: String,
    ends_at: String,
    capacity: u32,
    poster_upload_key: Option<String>,
}

pub async fn handle(
    req: Request,
    repo: &dyn EventsRepo,
    clock: &dyn Clock,
    storage: &dyn PosterStorage,
    auth: &dyn AuthService,
) -> HttpResponse {
    if let Err(resp) = require_admin(&req, auth) {
        return resp;
    }

    let body: Body = match json_body(&req) {
        Ok(b) => b,
        Err(resp) => return resp,
    };

    let starts_at = match DateTime::parse_from_rfc3339(&body.starts_at) {
        Ok(dt) => dt.with_timezone(&chrono::Utc),
        Err(_) => return error_response(AppError::InvalidEvent(EventError::InvalidDateRange)),
    };
    let ends_at = match DateTime::parse_from_rfc3339(&body.ends_at) {
        Ok(dt) => dt.with_timezone(&chrono::Utc),
        Err(_) => return error_response(AppError::InvalidEvent(EventError::InvalidDateRange)),
    };

    let poster_url = body
        .poster_upload_key
        .as_deref()
        .map(|k| storage.public_url(k));

    let input = CreateEventInput {
        id: EventId::new(format!("evt_{}", Uuid::new_v4().simple())),
        name: body.name,
        description: body.description,
        poster_url,
        location: body.location,
        starts_at,
        ends_at,
        capacity: body.capacity,
    };

    match create_event(repo, clock, input).await {
        Ok(event) => created(json!({
            "event_id":         event.id.as_str(),
            "name":             event.name,
            "description":      event.description,
            "location":         event.location,
            "starts_at":        event.starts_at.to_rfc3339(),
            "ends_at":          event.ends_at.to_rfc3339(),
            "capacity":         event.capacity,
            "registered_count": 0,
            "poster_url":       event.poster_url,
            "created_at":       event.created_at.to_rfc3339(),
        })),
        Err(e) => error_response(e),
    }
}
