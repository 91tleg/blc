use crate::application::{errors::AppError, services::AuthService};
use crate::infrastructure::jwt::JwtService;
use argon2::{
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use password_hash::rand_core::OsRng;

pub struct AdminAuthService {
    admin_password_hash: String,
    jwt: JwtService,
}

impl AdminAuthService {
    pub fn new(admin_password: &str, jwt_secret: &str) -> Self {
        // Hash the password immediately upon initialization
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let password_hash = argon2
            .hash_password(admin_password.as_bytes(), &salt)
            .expect("Failed to hash admin password")
            .to_string();

        Self {
            admin_password_hash: password_hash,
            jwt: JwtService::new(jwt_secret),
        }
    }
}

impl AuthService for AdminAuthService {
    fn verify_admin_password(&self, password: &str) -> Result<(), AppError> {
        let parsed_hash = PasswordHash::new(&self.admin_password_hash)
            .map_err(|_| AppError::AuthError("Internal authentication error".into()))?;

        // Argon2 handles constant-time comparison internally to prevent timing attacks
        Argon2::default()
            .verify_password(password.as_bytes(), &parsed_hash)
            .map_err(|_| AppError::AuthError("invalid credentials".into()))
    }

    fn issue_admin_token(&self) -> Result<String, AppError> {
        self.jwt.issue_admin_token()
    }

    fn verify_admin_token(&self, token: &str) -> Result<(), AppError> {
        self.jwt.verify(token).map(|_| ())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_JWT_SECRET: &str = "test_secret_12345678901234567890123456789012";
    const ADMIN_PW: &str = "super-secret-admin-pass";

    fn setup() -> AdminAuthService {
        AdminAuthService::new(ADMIN_PW, TEST_JWT_SECRET)
    }

    #[test]
    fn test_verify_password_success() {
        let service = setup();
        assert!(service.verify_admin_password(ADMIN_PW).is_ok());
    }

    #[test]
    fn test_verify_password_failure() {
        let service = setup();
        let result = service.verify_admin_password("wrong-password");
        assert!(result.is_err());

        if let Err(AppError::AuthError(msg)) = result {
            assert_eq!(msg, "invalid credentials");
        } else {
            panic!("Expected AuthError");
        }
    }

    #[test]
    fn test_empty_password_failure() {
        let service = setup();
        assert!(service.verify_admin_password("").is_err());
    }

    #[test]
    fn test_hash_uniqueness() {
        // Ensure that two services with the same password have different hashes
        let service1 = setup();
        let service2 = setup();
        assert_ne!(service1.admin_password_hash, service2.admin_password_hash);
    }
}
