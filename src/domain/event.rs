use chrono::{DateTime, Utc};
use std::fmt;

use crate::domain::types::EventId;

#[derive(Debug, Clone)]
pub struct Event {
    pub id: EventId,
    pub name: String,
    pub description: Option<String>,
    pub poster_url: Option<String>,
    pub location: String,
    pub starts_at: DateTime<Utc>,
    pub ends_at: DateTime<Utc>,
    pub capacity: u32,
    pub created_at: DateTime<Utc>,
}

impl Event {
    pub fn new(
        id: EventId,
        name: String,
        description: Option<String>,
        poster_url: Option<String>,
        location: String,
        starts_at: DateTime<Utc>,
        ends_at: DateTime<Utc>,
        capacity: u32,
        created_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            name,
            description,
            poster_url,
            location,
            starts_at,
            ends_at,
            capacity,
            created_at,
        }
    }

    pub fn validate(&self) -> Result<(), EventError> {
        if self.name.trim().is_empty() {
            return Err(EventError::InvalidName);
        }
        if self.name.len() > 100 {
            return Err(EventError::NameTooLong);
        }
        if self.location.trim().is_empty() {
            return Err(EventError::InvalidLocation);
        }
        if self.capacity == 0 {
            return Err(EventError::InvalidCapacity);
        }
        if self.ends_at <= self.starts_at {
            return Err(EventError::InvalidDateRange);
        }
        Ok(())
    }

    pub fn is_full(&self, registered_count: u32) -> bool {
        registered_count >= self.capacity
    }
}

#[derive(Debug)]
pub enum EventError {
    InvalidName,
    NameTooLong,
    InvalidLocation,
    InvalidCapacity,
    InvalidDateRange,
}

impl fmt::Display for EventError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EventError::InvalidName => write!(f, "Event name cannot be empty"),
            EventError::NameTooLong => write!(f, "Event name cannot exceed 100 characters"),
            EventError::InvalidLocation => write!(f, "Event location cannot be empty"),
            EventError::InvalidCapacity => write!(f, "Event capacity must be greater than zero"),
            EventError::InvalidDateRange => write!(f, "Event must end after it starts"),
        }
    }
}

impl std::error::Error for EventError {}
