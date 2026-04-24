use std::sync::Arc;

use aws_config::BehaviorVersion;
use lambda_http::{run, service_fn, Error, Request};
use tracing_subscriber::EnvFilter;

mod application;
mod domain;
mod handlers;
mod http_handler;
mod infrastructure;
mod utils;

use infrastructure::{
    auth::AdminAuthService,
    clock::SystemClock,
    dynamo::{events_repo::DynamoEventsRepo, registrations_repo::DynamoRegistrationsRepo},
    email::SesRegistrationEmailSender,
    s3::poster_storage::S3PosterStorage,
};

struct AppState {
    events_repo: DynamoEventsRepo,
    registrations_repo: DynamoRegistrationsRepo,
    clock: SystemClock,
    storage: S3PosterStorage,
    auth: AdminAuthService,
    email_sender: SesRegistrationEmailSender,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .json()
        .with_target(false)
        .with_current_span(false)
        .init();

    // Non-secret infrastructure values are injected by SAM / CloudFormation.
    // Secrets are loaded from SSM SecureString parameters during cold start.

    let events_table = require_env("EVENTS_TABLE");
    let registrations_table = require_env("REGISTRATIONS_TABLE");
    let registrations_email_gsi = require_env("REGISTRATIONS_EMAIL_GSI");
    let posters_bucket = require_env("POSTERS_BUCKET");
    let posters_public_base_url = require_env("POSTERS_PUBLIC_BASE_URL");
    let registration_email_from_address = optional_env("REGISTRATION_EMAIL_FROM_ADDRESS");
    let admin_password_param = require_env("ADMIN_PASSWORD_PARAM");
    let jwt_secret_param = require_env("JWT_SECRET_PARAM");

    let aws_config = aws_config::load_defaults(BehaviorVersion::latest()).await;
    let ssm_client = aws_sdk_ssm::Client::new(&aws_config);
    let dynamo_client = aws_sdk_dynamodb::Client::new(&aws_config);
    let s3_client = aws_sdk_s3::Client::new(&aws_config);
    let ses_client = aws_sdk_sesv2::Client::new(&aws_config);
    let (admin_password, jwt_secret) = tokio::try_join!(
        load_secure_parameter(&ssm_client, &admin_password_param),
        load_secure_parameter(&ssm_client, &jwt_secret_param),
    )?;

    let state = Arc::new(AppState {
        events_repo: DynamoEventsRepo::new(dynamo_client.clone(), &events_table),
        registrations_repo: DynamoRegistrationsRepo::new(
            dynamo_client,
            &registrations_table,
            &registrations_email_gsi,
            &events_table,
        ),
        clock: SystemClock,
        storage: S3PosterStorage::new(s3_client, &posters_bucket, &posters_public_base_url),
        auth: AdminAuthService::new(&admin_password, &jwt_secret),
        email_sender: SesRegistrationEmailSender::new(
            ses_client,
            registration_email_from_address,
        ),
    });

    tracing::info!("BLC Lambda cold start complete");

    run(service_fn(|req: Request| {
        let state = Arc::clone(&state);
        async move {
            let resp = http_handler::dispatch(
                req,
                &state.events_repo,
                &state.registrations_repo,
                &state.email_sender,
                &state.clock,
                &state.storage,
                &state.auth,
            )
            .await;
            // lambda_http requires Ok(_) — errors are expressed as HTTP responses,
            // never as Err(). The Ok wrapper is purely for the runtime protocol.
            Ok::<_, Error>(resp)
        }
    }))
    .await
}

/// Reads a required environment variable. Panics with a clear message if missing.
/// Intentionally panics — a missing env var at cold start is a deployment error,
/// not a recoverable runtime condition.
fn require_env(key: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| panic!("required environment variable {key} is not set"))
}

fn optional_env(key: &str) -> Option<String> {
    std::env::var(key)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

async fn load_secure_parameter(
    client: &aws_sdk_ssm::Client,
    name: &str,
) -> Result<String, Error> {
    let output = client
        .get_parameter()
        .name(name)
        .with_decryption(true)
        .send()
        .await?;

    let parameter = output.parameter().ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("SSM parameter {name} was not returned"),
        )
    })?;

    let value = parameter.value().ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("SSM parameter {name} did not contain a value"),
        )
    })?;

    Ok(value.to_owned())
}
