use lambda_http::Request;
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

use crate::application::{
    ports::{EventsRepo, RegistrationsRepo},
    register_for_event::{register_for_event, RegisterForEventInput},
    services::{Clock, RegistrationEmailSender},
};
use crate::domain::types::{EventId, RegistrationId};
use crate::utils::{
    parse::{json_body, path_param},
    response::{created, error_response, HttpResponse},
};

#[derive(Deserialize)]
struct Body {
    full_name: String,
    email: String,
    phone_number: Option<String>,
    date_key: Option<String>,
}

pub async fn handle(
    req: Request,
    events_repo: &dyn EventsRepo,
    registrations_repo: &dyn RegistrationsRepo,
    email_sender: &dyn RegistrationEmailSender,
    clock: &dyn Clock,
) -> HttpResponse {
    let event_id = match path_param(&req, "event_id") {
        Ok(id) => EventId::new(id),
        Err(resp) => return resp,
    };

    let body: Body = match json_body(&req) {
        Ok(b) => b,
        Err(resp) => return resp,
    };

    let input = RegisterForEventInput {
        registration_id: RegistrationId::new(format!("reg_{}", Uuid::new_v4().simple())),
        event_id,
        full_name: body.full_name,
        email: body.email,
        phone_number: body.phone_number,
        date_key: normalize_date_key(body.date_key),
    };

    match register_for_event(events_repo, registrations_repo, email_sender, clock, input).await {
        Ok(reg) => created(json!({
            "registration_id": reg.id.as_str(),
            "event_id":        reg.event_id.as_str(),
            "full_name":       reg.full_name,
            "email":           reg.email,
            "phone_number":    reg.phone_number,
            "date_key":        reg.date_key,
            "registered_at":   reg.registered_at.to_rfc3339(),
        })),
        Err(e) => error_response(e),
    }
}

fn normalize_date_key(date_key: Option<String>) -> Option<String> {
    date_key.filter(|value| {
        value.len() == 10
            && value.chars().enumerate().all(|(index, c)| {
                matches!(index, 4 | 7) == (c == '-') && (c.is_ascii_digit() || c == '-')
            })
    })
}
