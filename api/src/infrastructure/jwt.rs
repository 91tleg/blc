use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

use crate::application::errors::AppError;

const ALGORITHM: Algorithm = Algorithm::HS256;
const ADMIN_TOKEN_HOURS: i64 = 8;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // Subject — always "admin"
    pub role: String,
    pub exp: usize, // Expiry as a Unix timestamp
}

pub struct JwtService {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
}

impl JwtService {
    pub fn new(secret: &str) -> Self {
        Self {
            encoding_key: EncodingKey::from_secret(secret.as_bytes()),
            decoding_key: DecodingKey::from_secret(secret.as_bytes()),
        }
    }

    /// Issues a signed admin token valid for 8 hours.
    pub fn issue_admin_token(&self) -> Result<String, AppError> {
        let exp = Utc::now()
            .checked_add_signed(Duration::hours(ADMIN_TOKEN_HOURS))
            .ok_or_else(|| AppError::AuthError("timestamp overflow".into()))?
            .timestamp() as usize;

        let claims = Claims {
            sub: "admin".into(),
            role: "admin".into(),
            exp,
        };

        encode(&Header::new(ALGORITHM), &claims, &self.encoding_key)
            .map_err(|e| AppError::AuthError(format!("token sign error: {e}")))
    }

    /// Verifies a token and returns its claims. Rejects expired tokens.
    pub fn verify(&self, token: &str) -> Result<Claims, AppError> {
        let mut validation = Validation::new(ALGORITHM);
        validation.set_required_spec_claims(&["exp", "sub"]);

        decode::<Claims>(token, &self.decoding_key, &validation)
            .map(|data| data.claims)
            .map_err(|e| AppError::AuthError(format!("invalid token: {e}")))
    }
}
