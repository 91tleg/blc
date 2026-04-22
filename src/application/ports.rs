use crate::application::errors::AppError;
use crate::application::views::EventSummary;
use crate::domain::{
    event::Event,
    registration::Registration,
    types::{EventId, RegistrationId},
};

#[async_trait::async_trait]
pub trait EventsRepo: Send + Sync {
    async fn save(&self, event: &Event) -> Result<(), AppError>;

    async fn find_by_id(&self, id: &EventId) -> Result<Option<Event>, AppError>;

    /// Returns a page of event summaries (includes registered_count as a
    /// read-model value) plus an opaque cursor for the next page.
    async fn list(
        &self,
        limit: u32,
        cursor: Option<String>,
    ) -> Result<(Vec<EventSummary>, Option<String>), AppError>;

    async fn registered_count(&self, id: &EventId) -> Result<u32, AppError>;
}

#[async_trait::async_trait]
pub trait RegistrationsRepo: Send + Sync {
    async fn save(&self, registration: &Registration) -> Result<(), AppError>;

    async fn find_by_id(&self, id: &RegistrationId) -> Result<Option<Registration>, AppError>;

    async fn find_by_event_and_email(
        &self,
        event_id: &EventId,
        email: &str,
    ) -> Result<Option<Registration>, AppError>;

    /// Returns a page of registrations plus cursor and total count.
    async fn list_by_event(
        &self,
        event_id: &EventId,
        limit: u32,
        cursor: Option<String>,
    ) -> Result<(Vec<Registration>, Option<String>, u32), AppError>;
}
