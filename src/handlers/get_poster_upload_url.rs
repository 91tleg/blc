use lambda_http::Request;
use serde_json::json;
use uuid::Uuid;

use crate::application::{
    get_poster_upload_url::{get_poster_upload_url, PosterUploadUrlInput},
    services::{AuthService, PosterStorage},
};
use crate::utils::{
    auth_guard::require_admin,
    parse::required_query_param,
    response::{error_response, ok, HttpResponse},
};

const PRESIGN_EXPIRY_SECS: u32 = 300;

pub async fn handle(
    req: Request,
    storage: &dyn PosterStorage,
    auth: &dyn AuthService,
) -> HttpResponse {
    if let Err(resp) = require_admin(&req, auth) {
        return resp;
    }

    let filename = match required_query_param(&req, "filename") {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let content_type = match required_query_param(&req, "content_type") {
        Ok(v) => v,
        Err(resp) => return resp,
    };

    let upload_key = format!("posters/{}_{}", Uuid::new_v4().simple(), filename);

    let input = PosterUploadUrlInput {
        upload_key,
        content_type: content_type,
        expires_in_secs: PRESIGN_EXPIRY_SECS,
    };

    match get_poster_upload_url(storage, input) {
        Ok(out) => ok(json!({
            "upload_url":        out.upload_url,
            "poster_upload_key": out.poster_upload_key,
            "expires_in":        PRESIGN_EXPIRY_SECS,
        })),
        Err(e) => error_response(e),
    }
}
