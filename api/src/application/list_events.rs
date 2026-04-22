use crate::application::{errors::AppError, ports::EventsRepo, views::EventSummary};

pub struct ListEventsInput {
    pub limit: u32,
    pub cursor: Option<String>,
}

pub struct ListEventsOutput {
    pub events: Vec<EventSummary>,
    pub next_cursor: Option<String>,
}

pub async fn list_events(
    repo: &dyn EventsRepo,
    input: ListEventsInput,
) -> Result<ListEventsOutput, AppError> {
    let (events, next_cursor) = repo.list(input.limit, input.cursor).await?;
    Ok(ListEventsOutput {
        events,
        next_cursor,
    })
}
