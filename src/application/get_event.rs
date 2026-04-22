use crate::application::{errors::AppError, ports::EventsRepo, views::EventSummary};
use crate::domain::types::EventId;

pub async fn get_event(repo: &dyn EventsRepo, id: &EventId) -> Result<EventSummary, AppError> {
    let event = repo.find_by_id(id).await?.ok_or(AppError::EventNotFound)?;

    let registered_count = repo.registered_count(id).await?;

    Ok(EventSummary {
        id: event.id,
        name: event.name,
        description: event.description,
        poster_url: event.poster_url,
        location: event.location,
        starts_at: event.starts_at,
        ends_at: event.ends_at,
        capacity: event.capacity,
        registered_count,
        created_at: event.created_at,
    })
}
