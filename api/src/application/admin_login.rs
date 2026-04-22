use crate::application::{errors::AppError, services::AuthService};

pub struct AdminLoginInput<'a> {
    pub password: &'a str,
}

pub struct AdminLoginOutput {
    pub token: String,
}

pub fn admin_login(
    auth: &dyn AuthService,
    input: AdminLoginInput<'_>,
) -> Result<AdminLoginOutput, AppError> {
    auth.verify_admin_password(input.password)?;
    let token = auth.issue_admin_token()?;
    Ok(AdminLoginOutput { token })
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockAuthService {
        verify_password_returns_error: bool,
        issue_token_returns_error: bool,
        issued_token: String,
    }

    impl MockAuthService {
        fn new() -> Self {
            Self {
                verify_password_returns_error: false,
                issue_token_returns_error: false,
                issued_token: "mock-token-12345".to_string(),
            }
        }

        fn with_verify_error(mut self) -> Self {
            self.verify_password_returns_error = true;
            self
        }

        fn with_issue_error(mut self) -> Self {
            self.issue_token_returns_error = true;
            self
        }

        fn with_token(mut self, token: String) -> Self {
            self.issued_token = token;
            self
        }
    }

    impl AuthService for MockAuthService {
        fn verify_admin_password(&self, _password: &str) -> Result<(), AppError> {
            if self.verify_password_returns_error {
                Err(AppError::AuthError("Invalid password".to_string()))
            } else {
                Ok(())
            }
        }

        fn issue_admin_token(&self) -> Result<String, AppError> {
            if self.issue_token_returns_error {
                Err(AppError::AuthError("Token generation failed".to_string()))
            } else {
                Ok(self.issued_token.clone())
            }
        }

        fn verify_admin_token(&self, _token: &str) -> Result<(), AppError> {
            Ok(())
        }
    }

    #[test]
    fn test_admin_login_success() {
        let auth = MockAuthService::new();
        let input = AdminLoginInput {
            password: "correct-password",
        };

        let result = admin_login(&auth, input);

        assert!(result.is_ok());
        let output = result.unwrap();
        assert_eq!(output.token, "mock-token-12345");
    }

    #[test]
    fn test_admin_login_invalid_password() {
        let auth = MockAuthService::new().with_verify_error();
        let input = AdminLoginInput {
            password: "wrong-password",
        };

        let result = admin_login(&auth, input);

        assert!(result.is_err());
        match result {
            Err(AppError::AuthError(msg)) => assert_eq!(msg, "Invalid password"),
            _ => panic!("Expected AuthError"),
        }
    }

    #[test]
    fn test_admin_login_token_generation_failure() {
        let auth = MockAuthService::new().with_issue_error();
        let input = AdminLoginInput {
            password: "correct-password",
        };

        let result = admin_login(&auth, input);

        assert!(result.is_err());
        match result {
            Err(AppError::AuthError(msg)) => assert_eq!(msg, "Token generation failed"),
            _ => panic!("Expected AuthError"),
        }
    }

    #[test]
    fn test_admin_login_with_custom_token() {
        let custom_token = "custom-jwt-token-xyz".to_string();
        let auth = MockAuthService::new().with_token(custom_token.clone());
        let input = AdminLoginInput {
            password: "correct-password",
        };

        let result = admin_login(&auth, input);

        assert!(result.is_ok());
        let output = result.unwrap();
        assert_eq!(output.token, custom_token);
    }
}
