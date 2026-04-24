use chrono::{DateTime, Duration, Utc};

use crate::application::{
    errors::AppError,
    ports::{EventsRepo, RegistrationsRepo},
    services::{Clock, RegistrationEmailSender},
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
    email_sender: &dyn RegistrationEmailSender,
    clock: &dyn Clock,
    input: RegisterForEventInput,
) -> Result<Registration, AppError> {
    let registered_at = clock.now();
    let date_key = input
        .date_key
        .unwrap_or_else(|| date_key_for_registered_at(registered_at.clone()));
    let registration = Registration::new(
        input.registration_id,
        input.event_id.clone(),
        input.full_name,
        input.email.clone(),
        input.phone_number,
        Some(date_key.clone()),
        registered_at,
    );
    registration.validate()?;

    let event = events_repo
        .find_by_id(&input.event_id)
        .await?
        .ok_or(AppError::EventNotFound)?;

    if let Some(phone_number) = &registration.phone_number {
        if has_same_date_email_phone(
            registrations_repo,
            &input.event_id,
            &date_key,
            &registration.email,
            phone_number,
        )
        .await?
        {
            return Err(AppError::AlreadyRegistered);
        }
    }

    let registered_count = events_repo.registered_count(&input.event_id).await?;
    if event.is_full(registered_count) {
        return Err(AppError::EventFull);
    }

    registrations_repo.save(&registration).await?;

    if let Err(error) = email_sender
        .send_registration_confirmation(&registration, &event)
        .await
    {
        tracing::error!(
            error = %error,
            event_id = %registration.event_id,
            registration_id = %registration.id,
            email = %registration.email,
            "failed to send registration confirmation email"
        );
    }

    Ok(registration)
}

async fn has_same_date_email_phone(
    registrations_repo: &dyn RegistrationsRepo,
    event_id: &EventId,
    date_key: &str,
    email: &str,
    phone_number: &str,
) -> Result<bool, AppError> {
    let target_phone = digits_only(phone_number);
    if target_phone.is_empty() {
        return Ok(false);
    }

    let mut cursor = None;

    loop {
        let (registrations, next_cursor, _) = registrations_repo
            .list_by_event(event_id, 200, cursor)
            .await?;

        let has_duplicate = registrations.iter().any(|registration| {
            let existing_date_key = registration
                .date_key
                .clone()
                .unwrap_or_else(|| date_key_for_registered_at(registration.registered_at.clone()));
            let existing_phone = registration
                .phone_number
                .as_deref()
                .map(digits_only)
                .unwrap_or_default();

            existing_date_key == date_key
                && registration.email.trim().eq_ignore_ascii_case(email.trim())
                && existing_phone == target_phone
        });

        if has_duplicate {
            return Ok(true);
        }

        match next_cursor {
            Some(next) => cursor = Some(next),
            None => return Ok(false),
        }
    }
}

fn digits_only(value: &str) -> String {
    value.chars().filter(|c| c.is_ascii_digit()).collect()
}

fn date_key_for_registered_at(registered_at: DateTime<Utc>) -> String {
    (registered_at - Duration::hours(7))
        .format("%Y-%m-%d")
        .to_string()
}
