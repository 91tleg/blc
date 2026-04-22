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
    s3::poster_storage::S3PosterStorage,
};

struct AppState {
    events_repo: DynamoEventsRepo,
    registrations_repo: DynamoRegistrationsRepo,
    clock: SystemClock,
    storage: S3PosterStorage,
    auth: AdminAuthService,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .json()
        .with_target(false)
        .with_current_span(false)
        .init();

    // All values are injected by SAM / CloudFormation at deploy time.
    // Panic at cold start if any are missing — better to fail fast than to
    // serve requests with a broken config.

    let events_table = require_env("EVENTS_TABLE");
    let registrations_table = require_env("REGISTRATIONS_TABLE");
    let registrations_email_gsi = require_env("REGISTRATIONS_EMAIL_GSI");
    let posters_bucket = require_env("POSTERS_BUCKET");
    let posters_public_base_url = require_env("POSTERS_PUBLIC_BASE_URL");
    let admin_password = require_env("ADMIN_PASSWORD");
    let jwt_secret = require_env("JWT_SECRET");

    let aws_config = aws_config::load_defaults(BehaviorVersion::latest()).await;
    let dynamo_client = aws_sdk_dynamodb::Client::new(&aws_config);
    let s3_client = aws_sdk_s3::Client::new(&aws_config);

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
    });

    tracing::info!("BLC Lambda cold start complete");

    run(service_fn(|req: Request| {
        let state = Arc::clone(&state);
        async move {
            let resp = http_handler::dispatch(
                req,
                &state.events_repo,
                &state.registrations_repo,
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
