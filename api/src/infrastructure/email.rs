use aws_sdk_sesv2::{
    types::{Body, Content, Destination, EmailContent, Message},
    Client,
};
use chrono::NaiveDate;

use crate::{
    application::{errors::AppError, services::RegistrationEmailSender},
    domain::{event::Event, registration::Registration},
};

pub struct SesRegistrationEmailSender {
    client: Client,
    from_address: Option<String>,
}

impl SesRegistrationEmailSender {
    pub fn new(client: Client, from_address: Option<String>) -> Self {
        if from_address.is_none() {
            tracing::warn!(
                "registration email sender is not configured; confirmation emails are disabled"
            );
        }

        Self {
            client,
            from_address,
        }
    }
}

#[async_trait::async_trait]
impl RegistrationEmailSender for SesRegistrationEmailSender {
    async fn send_registration_confirmation(
        &self,
        registration: &Registration,
        event: &Event,
    ) -> Result<(), AppError> {
        let from_address = match &self.from_address {
            Some(value) => value,
            None => return Ok(()),
        };

        let event_name = normalized_event_name(event);
        let subject = format!("You are signed up for {event_name}");
        let text_body = build_text_body(registration, event, &event_name);
        let html_body = build_html_body(registration, event, &event_name);

        let subject_content = email_content(&subject)?;
        let text_content = email_content(&text_body)?;
        let html_content = email_content(&html_body)?;
        let message_body = Body::builder()
            .text(text_content)
            .html(html_content)
            .build()
            .map_err(|e| AppError::StorageError(format!("failed to build email body: {e}")))?;
        let message = Message::builder()
            .subject(subject_content)
            .body(message_body)
            .build()
            .map_err(|e| AppError::StorageError(format!("failed to build email message: {e}")))?;
        let destination = Destination::builder()
            .to_addresses(registration.email.clone())
            .build();

        let response = self
            .client
            .send_email()
            .from_email_address(from_address)
            .destination(destination)
            .content(EmailContent::builder().simple(message).build())
            .send()
            .await
            .map_err(|e| AppError::StorageError(format!("failed to send email: {e}")))?;

        tracing::info!(
            email = %registration.email,
            event_id = %registration.event_id,
            registration_id = %registration.id,
            message_id = response.message_id().unwrap_or_default(),
            "registration confirmation email queued"
        );

        Ok(())
    }
}

fn email_content(value: &str) -> Result<Content, AppError> {
    Content::builder()
        .charset("UTF-8")
        .data(value)
        .build()
        .map_err(|e| AppError::StorageError(format!("failed to build email content: {e}")))
}

fn normalized_event_name(event: &Event) -> String {
    let trimmed = event.name.trim();
    if trimmed.is_empty() {
        "the event".to_string()
    } else {
        trimmed.to_string()
    }
}

fn build_text_body(registration: &Registration, event: &Event, event_name: &str) -> String {
    let greeting_name = first_name(&registration.full_name);
    let event_date = formatted_registration_date(registration);
    let location_line = if event.location.trim().is_empty() {
        String::new()
    } else {
        format!("Location: {}\n", event.location.trim())
    };

    format!(
        "Hi {greeting_name},\n\nYou have been signed up for {event_name}.\n\n{event_date}{location_line}\nIf you did not expect this registration, please contact the event organizer.\n\nBusiness Leadership Community"
    )
}

fn build_html_body(registration: &Registration, event: &Event, event_name: &str) -> String {
    let greeting_name = html_escape(&first_name(&registration.full_name));
    let safe_event_name = html_escape(event_name);
    let event_date = formatted_registration_date(registration);
    let location = event.location.trim();
    let location_row = if location.is_empty() {
        String::new()
    } else {
        format!(
            "<p style=\"margin:0 0 10px;\"><strong>Location:</strong> {}</p>",
            html_escape(location)
        )
    };

    format!(
        "<!doctype html><html><body style=\"font-family:Arial,sans-serif;color:#111;line-height:1.5;\"><p>Hi {greeting_name},</p><p>You have been signed up for <strong>{safe_event_name}</strong>.</p><p style=\"margin:0 0 10px;\"><strong>{}</strong></p>{location_row}<p>If you did not expect this registration, please contact the event organizer.</p><p>Business Leadership Community</p></body></html>",
        html_escape(&event_date)
    )
}

fn first_name(full_name: &str) -> String {
    full_name
        .split_whitespace()
        .next()
        .filter(|value| !value.is_empty())
        .unwrap_or("there")
        .to_string()
}

fn formatted_registration_date(registration: &Registration) -> String {
    registration
        .date_key
        .as_deref()
        .and_then(|value| NaiveDate::parse_from_str(value, "%Y-%m-%d").ok())
        .map(|date| format!("Event date: {}", date.format("%A, %B %-d, %Y")))
        .unwrap_or_else(|| "Event date: To be announced".to_string())
}

fn html_escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}
