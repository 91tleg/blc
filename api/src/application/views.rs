use chrono::{DateTime, Utc};

use crate::domain::types::EventId;

#[derive(Debug, Clone)]
pub struct EventSummary {
    pub id: EventId,
    pub name: String,
    pub description: Option<String>,
    pub poster_url: Option<String>,
    pub location: String,
    pub starts_at: DateTime<Utc>,
    pub ends_at: DateTime<Utc>,
    pub capacity: u32,
    pub registered_count: u32,
    pub created_at: DateTime<Utc>,
}
