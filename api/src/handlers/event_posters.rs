use base64::{engine::general_purpose::STANDARD, Engine};
use chrono::Utc;
use lambda_http::Request;
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

use crate::application::{
    ports::EventsRepo,
    views::PosterSummary,
};
use crate::domain::types::EventId;
use crate::infrastructure::s3::poster_storage::S3PosterStorage;
use crate::utils::{
    parse::{json_body, path_param},
    response::{created, error_response, ok, HttpResponse},
};

#[derive(Deserialize)]
struct SavePosterBody {
    name: String,
    data_url: String,
    date_key: Option<String>,
}

pub async fn list(req: Request, repo: &dyn EventsRepo) -> HttpResponse {
    let event_id = match event_id_from_path(&req) {
        Ok(id) => id,
        Err(resp) => return resp,
    };

    match repo.list_posters(&event_id).await {
        Ok(posters) => ok(json!({
            "posters": posters.iter().map(poster_json).collect::<Vec<_>>()
        })),
        Err(e) => error_response(e),
    }
}

pub async fn save(
    req: Request,
    repo: &dyn EventsRepo,
    storage: &S3PosterStorage,
) -> HttpResponse {
    let event_id = match event_id_from_path(&req) {
        Ok(id) => id,
        Err(resp) => return resp,
    };

    let body: SavePosterBody = match json_body(&req) {
        Ok(body) => body,
        Err(resp) => return resp,
    };

    let (content_type, bytes) = match decode_data_url(&body.data_url) {
        Ok(decoded) => decoded,
        Err(resp) => return resp,
    };

    let poster_id = format!("poster_{}", Uuid::new_v4().simple());
    let extension = match content_type.as_str() {
        "image/png" => "png",
        "image/webp" => "webp",
        _ => "jpg",
    };
    let object_key = format!(
        "event-posters/{}/{}.{}",
        event_id.as_str(),
        poster_id,
        extension
    );

    let url = match storage.put_object(&object_key, &content_type, bytes).await {
        Ok(url) => url,
        Err(e) => return error_response(e),
    };

    let poster = PosterSummary {
        id: poster_id,
        name: body.name.trim().chars().take(160).collect(),
        url,
        object_key,
        date_key: normalize_date_key(body.date_key),
        uploaded_at: Utc::now(),
    };

    match repo.save_poster(&event_id, &poster).await {
        Ok(()) => created(poster_json(&poster)),
        Err(e) => error_response(e),
    }
}

pub async fn delete(
    req: Request,
    repo: &dyn EventsRepo,
    storage: &S3PosterStorage,
) -> HttpResponse {
    let event_id = match event_id_from_path(&req) {
        Ok(id) => id,
        Err(resp) => return resp,
    };
    let poster_id = match path_param(&req, "poster_id") {
        Ok(id) => id,
        Err(resp) => return resp,
    };

    match repo.delete_poster(&event_id, &poster_id).await {
        Ok(Some(object_key)) => {
            if let Err(e) = storage.delete_object(&object_key).await {
                return error_response(e);
            }
            ok(json!({ "deleted": true }))
        }
        Ok(None) => ok(json!({ "deleted": false })),
        Err(e) => error_response(e),
    }
}

fn event_id_from_path(req: &Request) -> Result<EventId, HttpResponse> {
    path_param(req, "event_id").map(EventId::new)
}

fn poster_json(poster: &PosterSummary) -> serde_json::Value {
    json!({
        "poster_id": poster.id,
        "name": poster.name,
        "url": poster.url,
        "date_key": poster.date_key,
        "uploaded_at": poster.uploaded_at.to_rfc3339(),
    })
}

fn normalize_date_key(date_key: Option<String>) -> String {
    date_key
        .filter(|value| {
            value.len() == 10
                && value
                    .chars()
                    .enumerate()
                    .all(|(index, c)| matches!(index, 4 | 7) == (c == '-') && (c.is_ascii_digit() || c == '-'))
        })
        .unwrap_or_else(|| Utc::now().format("%Y-%m-%d").to_string())
}

fn decode_data_url(data_url: &str) -> Result<(String, Vec<u8>), HttpResponse> {
    let (metadata, payload) = data_url.split_once(',').ok_or_else(bad_poster)?;
    let content_type = metadata
        .strip_prefix("data:")
        .and_then(|value| value.strip_suffix(";base64"))
        .ok_or_else(bad_poster)?;

    if !matches!(content_type, "image/jpeg" | "image/png" | "image/webp") {
        return Err(bad_poster());
    }

    let bytes = STANDARD.decode(payload).map_err(|_| bad_poster())?;
    if bytes.is_empty() || bytes.len() > 8 * 1024 * 1024 {
        return Err(bad_poster());
    }

    Ok((content_type.to_string(), bytes))
}

fn bad_poster() -> HttpResponse {
    use lambda_http::{Body, Response};
    let body = serde_json::to_string(&json!({
        "error": {
            "code": "BAD_REQUEST",
            "message": "Poster must be a JPG, PNG, or WebP image under 8 MB"
        }
    }))
    .unwrap_or_default();

    Response::builder()
        .status(400)
        .header("Content-Type", "application/json")
        .header("Access-Control-Allow-Origin", "*")
        .body(Body::Text(body))
        .expect("failed to build bad poster response")
}
