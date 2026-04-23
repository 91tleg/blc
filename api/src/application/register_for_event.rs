use crate::application::{
    errors::AppError,
    ports::{EventsRepo, RegistrationsRepo},
    services::Clock,
};
use crate::domain::{
    registration::Registration,
    types::{EventId, RegistrationId},
};

pub struct RegisterForEventInput {
    pub registration_id: RegistrationId,
    pub event_id: EventId,
    pub full_name: String,
    pub email: String,
    pub phone_number: Option<String>,
    pub date_key: Option<String>,
}

pub async fn register_for_event(
    events_repo: &dyn EventsRepo,
    registrations_repo: &dyn RegistrationsRepo,
    clock: &dyn Clock,
    input: RegisterForEventInput,
) -> Result<Registration, AppError> {
    let registration = Registration::new(
        input.registration_id,
        input.event_id.clone(),
        input.full_name,
        input.email.clone(),
        input.phone_number,
        input.date_key,
        clock.now(),
    );
    registration.validate()?;

    let event = events_repo
        .find_by_id(&input.event_id)
        .await?
        .ok_or(AppError::EventNotFound)?;

    let registered_count = events_repo.registered_count(&input.event_id).await?;
    if event.is_full(registered_count) {
        return Err(AppError::EventFull);
    }

    registrations_repo.save(&registration).await?;

    Ok(registration)
}
