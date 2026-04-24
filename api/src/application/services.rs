use chrono::{DateTime, Utc};

use crate::application::errors::AppError;
use crate::domain::{event::Event, registration::Registration};

pub trait Clock: Send + Sync {
    fn now(&self) -> DateTime<Utc>;
}

pub trait PosterStorage: Send + Sync {
    /// Returns a pre-signed upload URL for the given object key.
    fn presign_upload_url(
        &self,
        key: &str,
        content_type: &str,
        expires_in_secs: u32,
    ) -> Result<String, AppError>;

    /// Resolves an object key to its public HTTPS URL.
    fn public_url(&self, key: &str) -> String;
}

pub trait AuthService: Send + Sync {
    fn verify_admin_password(&self, password: &str) -> Result<(), AppError>;
    fn issue_admin_token(&self) -> Result<String, AppError>;
    fn verify_admin_token(&self, token: &str) -> Result<(), AppError>;
}

#[async_trait::async_trait]
pub trait RegistrationEmailSender: Send + Sync {
    async fn send_registration_confirmation(
        &self,
        registration: &Registration,
        event: &Event,
    ) -> Result<(), AppError>;
}
