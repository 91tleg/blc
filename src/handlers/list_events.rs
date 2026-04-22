use lambda_http::Request;
use serde_json::json;

use crate::application::{
    list_events::{list_events, ListEventsInput},
    ports::EventsRepo,
};
use crate::utils::{
    parse::query_param,
    response::{error_response, ok, HttpResponse},
};

const DEFAULT_LIMIT: u32 = 20;
const MAX_LIMIT: u32 = 100;

pub async fn handle(req: Request, repo: &dyn EventsRepo) -> HttpResponse {
    let limit = query_param(&req, "limit")
        .and_then(|v| v.parse::<u32>().ok())
        .unwrap_or(DEFAULT_LIMIT)
        .min(MAX_LIMIT);

    let cursor = query_param(&req, "cursor");

    match list_events(repo, ListEventsInput { limit, cursor }).await {
        Ok(out) => ok(json!({
            "events": out.events.iter().map(|e| json!({
                "event_id":        e.id.as_str(),
                "name":            e.name,
                "description":     e.description,
                "location":        e.location,
                "starts_at":       e.starts_at.to_rfc3339(),
                "ends_at":         e.ends_at.to_rfc3339(),
                "capacity":        e.capacity,
                "registered_count": e.registered_count,
                "poster_url":      e.poster_url,
                "created_at":      e.created_at.to_rfc3339(),
            })).collect::<Vec<_>>(),
            "next_cursor": out.next_cursor,
        })),
        Err(e) => error_response(e),
    }
}
