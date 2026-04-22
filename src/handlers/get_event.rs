use lambda_http::Request;
use serde_json::json;

use crate::application::{get_event::get_event, ports::EventsRepo};
use crate::domain::types::EventId;
use crate::utils::{
    parse::path_param,
    response::{error_response, ok, HttpResponse},
};

pub async fn handle(req: Request, repo: &dyn EventsRepo) -> HttpResponse {
    let event_id = match path_param(&req, "event_id") {
        Ok(id) => EventId::new(id),
        Err(resp) => return resp,
    };

    match get_event(repo, &event_id).await {
        Ok(e) => ok(json!({
            "event_id":         e.id.as_str(),
            "name":             e.name,
            "description":      e.description,
            "location":         e.location,
            "starts_at":        e.starts_at.to_rfc3339(),
            "ends_at":          e.ends_at.to_rfc3339(),
            "capacity":         e.capacity,
            "registered_count": e.registered_count,
            "poster_url":       e.poster_url,
            "created_at":       e.created_at.to_rfc3339(),
        })),
        Err(e) => error_response(e),
    }
}
