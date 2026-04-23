use chrono::{DateTime, Utc};
use std::fmt;

use crate::domain::types::{EventId, RegistrationId};

#[derive(Debug, Clone)]
pub struct Registration {
    pub id: RegistrationId,
    pub event_id: EventId,
    pub full_name: String,
    pub email: String,
    pub phone_number: Option<String>,
    pub date_key: Option<String>,
    pub registered_at: DateTime<Utc>,
}

impl Registration {
    pub fn new(
        id: RegistrationId,
        event_id: EventId,
        full_name: String,
        email: String,
        phone_number: Option<String>,
        date_key: Option<String>,
        registered_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            event_id,
            full_name,
            email,
            phone_number,
            date_key,
            registered_at,
        }
    }

    pub fn validate(&self) -> Result<(), RegistrationError> {
        if self.full_name.trim().is_empty() {
            return Err(RegistrationError::InvalidName);
        }
        if !is_valid_email(&self.email) {
            return Err(RegistrationError::InvalidEmail);
        }
        if let Some(phone) = &self.phone_number {
            if !is_valid_phone(phone) {
                return Err(RegistrationError::InvalidPhone);
            }
        }
        Ok(())
    }
}

fn is_valid_email(email: &str) -> bool {
    match email.splitn(2, '@').collect::<Vec<_>>().as_slice() {
        [local, domain] => !local.is_empty() && domain.contains('.') && !domain.starts_with('.'),
        _ => false,
    }
}

fn is_valid_phone(phone: &str) -> bool {
    let digits: String = phone.chars().filter(|c| c.is_ascii_digit()).collect();
    digits.len() >= 10 && digits.len() <= 15
}

#[derive(Debug)]
pub enum RegistrationError {
    InvalidName,
    InvalidEmail,
    InvalidPhone,
}

impl fmt::Display for RegistrationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RegistrationError::InvalidName => write!(f, "Attendee name cannot be empty"),
            RegistrationError::InvalidEmail => write!(f, "A valid email address is required"),
            RegistrationError::InvalidPhone => write!(f, "Phone number must contain 10-15 digits"),
        }
    }
}

impl std::error::Error for RegistrationError {}
