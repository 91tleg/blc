use lambda_http::Request;
use serde::Deserialize;
use serde_json::json;

use crate::application::admin_login::{admin_login, AdminLoginInput};
use crate::application::services::AuthService;
use crate::utils::{
    parse::json_body,
    response::{created, error_response, HttpResponse},
};

#[derive(Deserialize)]
struct Body {
    password: String,
}

pub async fn handle(req: Request, auth: &dyn AuthService) -> HttpResponse {
    let body: Body = match json_body(&req) {
        Ok(b) => b,
        Err(resp) => return resp,
    };

    match admin_login(
        auth,
        AdminLoginInput {
            password: &body.password,
        },
    ) {
        Ok(out) => created(json!({ "token": out.token })),
        Err(e) => error_response(e),
    }
}
