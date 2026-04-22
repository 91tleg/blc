use aws_sdk_dynamodb::types::AttributeValue;
use base64::{engine::general_purpose::STANDARD, Engine};
use chrono::{DateTime, Utc};
use std::collections::HashMap;

use crate::application::errors::AppError;
use crate::domain::types::{EventId, RegistrationId};

pub mod events_repo;
pub mod registrations_repo;

/// Construct a string AttributeValue
pub fn s(value: &str) -> AttributeValue {
    AttributeValue::S(value.to_string())
}

/// Construct a number AttributeValue
pub fn n(value: u32) -> AttributeValue {
    AttributeValue::N(value.to_string())
}

/// Extract a string from an item, or return an error
pub fn get_s(item: &HashMap<String, AttributeValue>, key: &str) -> Result<String, AppError> {
    item.get(key)
        .and_then(|av| match av {
            AttributeValue::S(s) => Some(s.clone()),
            _ => None,
        })
        .ok_or_else(|| AppError::StorageError(format!("missing or invalid '{}'", key)))
}

/// Extract an optional string from an item
pub fn get_s_opt(
    item: &HashMap<String, AttributeValue>,
    key: &str,
) -> Result<Option<String>, AppError> {
    match item.get(key) {
        None => Ok(None),
        Some(AttributeValue::S(s)) => Ok(Some(s.clone())),
        Some(_) => Err(AppError::StorageError(format!(
            "invalid type for '{}'",
            key
        ))),
    }
}

/// Extract a number as u32 from an item
pub fn get_n_u32(item: &HashMap<String, AttributeValue>, key: &str) -> Result<u32, AppError> {
    item.get(key)
        .and_then(|av| match av {
            AttributeValue::N(n) => n.parse::<u32>().ok(),
            _ => None,
        })
        .ok_or_else(|| AppError::StorageError(format!("missing or invalid '{}'", key)))
}

/// Parse a timestamp string (RFC3339 format) from an item
pub fn parse_timestamp(
    item: &HashMap<String, AttributeValue>,
    key: &str,
) -> Result<DateTime<Utc>, AppError> {
    let s = get_s(item, key)?;
    s.parse::<DateTime<Utc>>()
        .map_err(|_| AppError::StorageError("invalid timestamp format".to_string()))
}

pub fn encode_cursor(key: &HashMap<String, AttributeValue>) -> Result<String, AppError> {
    let value: serde_json::Value = serde_dynamo::from_item(key.clone())
        .map_err(|e| AppError::StorageError(format!("failed to serialize key: {}", e)))?;

    let json = serde_json::to_string(&value)
        .map_err(|e| AppError::StorageError(format!("failed to encode cursor: {}", e)))?;

    Ok(STANDARD.encode(json))
}

pub fn decode_cursor(cursor: &str) -> Result<HashMap<String, AttributeValue>, AppError> {
    let bytes = STANDARD
        .decode(cursor)
        .map_err(|_| AppError::StorageError("invalid cursor encoding".to_string()))?;

    let value: serde_json::Value = serde_json::from_slice(&bytes)
        .map_err(|e| AppError::StorageError(format!("failed to parse cursor: {}", e)))?;

    let key: HashMap<String, AttributeValue> = serde_dynamo::to_item(value)
        .map_err(|e| AppError::StorageError(format!("failed to reconstruct key: {}", e)))?;

    Ok(key)
}

/// Event partition key: "EVENT#{event_id}"
pub fn event_pk(event_id: &EventId) -> String {
    format!("EVENT#{}", event_id)
}

/// Event data sort key: static "DATA" for the main event record
pub fn event_data_sk() -> &'static str {
    "DATA"
}

/// Event counter sort key for registration count
pub fn event_count_sk() -> &'static str {
    "COUNT"
}

/// Registration partition key: same as event pk
pub fn registration_pk(event_id: &EventId) -> String {
    event_pk(event_id)
}

/// Registration sort key: "REG#{registration_id}"
pub fn registration_sk(registration_id: &RegistrationId) -> String {
    format!("REG#{}", registration_id)
}

/// Registration email GSI sort key: "EMAIL#{email}"
pub fn registration_email_gsi_sk(email: &str) -> String {
    format!("EMAIL#{}", email)
}
