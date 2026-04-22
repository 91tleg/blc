use lambda_http::Request;
use serde_json::json;

use crate::application::{
    list_registrations::{list_registrations, ListRegistrationsInput},
    ports::{EventsRepo, RegistrationsRepo},
    services::AuthService,
};
use crate::domain::types::EventId;
use crate::utils::{
    auth_guard::require_admin,
    parse::{path_param, query_param},
    response::{error_response, ok, HttpResponse},
};

const DEFAULT_LIMIT: u32 = 50;
const MAX_LIMIT: u32 = 200;

pub async fn handle(
    req: Request,
    events_repo: &dyn EventsRepo,
    registrations_repo: &dyn RegistrationsRepo,
    auth: &dyn AuthService,
) -> HttpResponse {
    if let Err(resp) = require_admin(&req, auth) {
        return resp;
    }

    let event_id = match path_param(&req, "event_id") {
        Ok(id) => EventId::new(id),
        Err(resp) => return resp,
    };

    let limit = query_param(&req, "limit")
        .and_then(|v| v.parse::<u32>().ok())
        .unwrap_or(DEFAULT_LIMIT)
        .min(MAX_LIMIT);

    let cursor = query_param(&req, "cursor");

    let input = ListRegistrationsInput {
        event_id,
        limit,
        cursor,
    };

    match list_registrations(events_repo, registrations_repo, input).await {
        Ok(out) => ok(json!({
            "registrations": out.registrations.iter().map(|r| json!({
                "registration_id": r.id.as_str(),
                "full_name":       r.full_name,
                "email":           r.email,
                "phone_number":    r.phone_number,
                "registered_at":   r.registered_at.to_rfc3339(),
            })).collect::<Vec<_>>(),
            "next_cursor": out.next_cursor,
            "total":       out.total,
        })),
        Err(e) => error_response(e),
    }
}
