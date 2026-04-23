use crate::domain::{event::EventError, registration::RegistrationError};

#[derive(Debug)]
pub enum AppError {
    InvalidEvent(EventError),
    InvalidRegistration(RegistrationError),
    EventFull,
    AlreadyRegistered,
    EventNotFound,
    RegistrationNotFound,
    StorageError(String),
    AuthError(String),
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::InvalidEvent(e) => write!(f, "{e}"),
            AppError::InvalidRegistration(e) => write!(f, "{e}"),
            AppError::EventFull => write!(f, "This event has reached its capacity"),
            AppError::AlreadyRegistered => {
                write!(
                    f,
                    "This email and phone number are already registered for this date"
                )
            }
            AppError::EventNotFound => write!(f, "Event not found"),
            AppError::RegistrationNotFound => write!(f, "Registration not found"),
            AppError::StorageError(msg) => write!(f, "Storage error: {msg}"),
            AppError::AuthError(msg) => write!(f, "Auth error: {msg}"),
        }
    }
}

impl std::error::Error for AppError {}

impl From<EventError> for AppError {
    fn from(e: EventError) -> Self {
        AppError::InvalidEvent(e)
    }
}

impl From<RegistrationError> for AppError {
    fn from(e: RegistrationError) -> Self {
        AppError::InvalidRegistration(e)
    }
}
