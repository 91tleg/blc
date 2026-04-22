use crate::application::{
    errors::AppError,
    ports::{EventsRepo, RegistrationsRepo},
};
use crate::domain::{registration::Registration, types::EventId};

pub struct ListRegistrationsInput {
    pub event_id: EventId,
    pub limit: u32,
    pub cursor: Option<String>,
}

pub struct ListRegistrationsOutput {
    pub registrations: Vec<Registration>,
    pub next_cursor: Option<String>,
    pub total: u32,
}

pub async fn list_registrations(
    events_repo: &dyn EventsRepo,
    registrations_repo: &dyn RegistrationsRepo,
    input: ListRegistrationsInput,
) -> Result<ListRegistrationsOutput, AppError> {
    // Event must exist — return 404 rather than an empty list for unknown IDs
    events_repo
        .find_by_id(&input.event_id)
        .await?
        .ok_or(AppError::EventNotFound)?;

    let (registrations, next_cursor, total) = registrations_repo
        .list_by_event(&input.event_id, input.limit, input.cursor)
        .await?;

    Ok(ListRegistrationsOutput {
        registrations,
        next_cursor,
        total,
    })
}
