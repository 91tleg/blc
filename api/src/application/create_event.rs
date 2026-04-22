use chrono::{DateTime, Utc};

use crate::application::{errors::AppError, ports::EventsRepo, services::Clock};
use crate::domain::{event::Event, types::EventId};

pub struct CreateEventInput {
    pub id: EventId,
    pub name: String,
    pub description: Option<String>,
    pub poster_url: Option<String>,
    pub location: String,
    pub starts_at: DateTime<Utc>,
    pub ends_at: DateTime<Utc>,
    pub capacity: u32,
}

pub async fn create_event(
    repo: &dyn EventsRepo,
    clock: &dyn Clock,
    input: CreateEventInput,
) -> Result<Event, AppError> {
    let event = Event::new(
        input.id,
        input.name,
        input.description,
        input.poster_url,
        input.location,
        input.starts_at,
        input.ends_at,
        input.capacity,
        clock.now(),
    );

    event.validate()?;
    repo.save(&event).await?;
    Ok(event)
}
