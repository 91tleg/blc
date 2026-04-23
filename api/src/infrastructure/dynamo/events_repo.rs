use async_trait::async_trait;
use aws_sdk_dynamodb::{
    types::{AttributeValue, Put, TransactWriteItem},
    Client,
};
use futures::future::join_all;
use std::collections::HashMap;

use crate::application::{errors::AppError, ports::EventsRepo, views::EventSummary};
use crate::application::views::PosterSummary;
use crate::domain::{event::Event, types::EventId};
use crate::infrastructure::dynamo::{
    decode_cursor, encode_cursor, event_count_sk, event_data_sk, event_pk, get_n_u32, get_s,
    get_s_opt, n, parse_timestamp, s,
};

const POSTER_SK_PREFIX: &str = "POSTER#";

fn poster_sk(poster_id: &str) -> String {
    format!("{POSTER_SK_PREFIX}{poster_id}")
}

pub struct DynamoEventsRepo {
    pub client: Client,
    pub table_name: String,
}

impl DynamoEventsRepo {
    pub fn new(client: Client, table_name: impl Into<String>) -> Self {
        Self {
            client,
            table_name: table_name.into(),
        }
    }

    /// Optimized: Check for duplicate email using GSI before registering.
    pub async fn is_email_already_registered(
        &self,
        event_id: &EventId,
        email: &str,
    ) -> Result<bool, AppError> {
        let resp = self
            .client
            .query()
            .table_name(&self.table_name)
            .index_name("EmailIndex")
            .key_condition_expression("pk = :pk AND email = :email")
            // FIX: Borrow the result of event_pk
            .expression_attribute_values(":pk", s(&event_pk(event_id)))
            .expression_attribute_values(":email", s(&format!("EMAIL#{}", email)))
            .limit(1)
            .send()
            .await
            .map_err(|e| AppError::StorageError(e.to_string()))?;

        Ok(resp.count() > 0)
    }
}

#[async_trait]
impl EventsRepo for DynamoEventsRepo {
    async fn save(&self, event: &Event) -> Result<(), AppError> {
        // Build the Put for the Event Data
        let event_put = Put::builder()
            .table_name(&self.table_name)
            .set_item(Some(event_to_item(event)))
            .condition_expression("attribute_not_exists(pk)")
            .build()
            .map_err(|e| AppError::StorageError(e.to_string()))?;

        // Build the Put for the Counter
        let count_put = Put::builder()
            .table_name(&self.table_name)
            // FIX: Borrow the strings
            .item("pk", s(&event_pk(&event.id)))
            .item("sk", s(&event_count_sk()))
            .item("count", n(0u32))
            .condition_expression("attribute_not_exists(pk)")
            .build()
            .map_err(|e| AppError::StorageError(e.to_string()))?;

        self.client
            .transact_write_items()
            .transact_items(TransactWriteItem::builder().put(event_put).build())
            .transact_items(TransactWriteItem::builder().put(count_put).build())
            .send()
            .await
            .map_err(|e| AppError::StorageError(format!("Transaction failed: {}", e)))?;

        Ok(())
    }

    async fn find_by_id(&self, id: &EventId) -> Result<Option<Event>, AppError> {
        let resp = self
            .client
            .get_item()
            .table_name(&self.table_name)
            .key("pk", s(&event_pk(id)))
            .key("sk", s(event_data_sk()))
            .send()
            .await
            .map_err(|e| AppError::StorageError(e.to_string()))?;

        match resp.item {
            None => Ok(None),
            Some(item) => item_to_event(&item).map(Some),
        }
    }

    async fn list(
        &self,
        limit: u32,
        cursor: Option<String>,
    ) -> Result<(Vec<EventSummary>, Option<String>), AppError> {
        let mut req = self
            .client
            .scan()
            .table_name(&self.table_name)
            .filter_expression("sk = :sk")
            .expression_attribute_values(":sk", s(event_data_sk()))
            .limit(limit as i32);

        if let Some(c) = cursor {
            req = req.set_exclusive_start_key(Some(decode_cursor(&c)?));
        }

        let resp = req
            .send()
            .await
            .map_err(|e| AppError::StorageError(e.to_string()))?;

        let items = resp.items().to_vec();

        let futures = items.into_iter().map(|item| async move {
            let event_id = EventId::new(get_s(&item, "event_id")?);
            let count = self.registered_count(&event_id).await?;
            let event = item_to_event(&item)?;

            Ok(EventSummary {
                id: event.id,
                name: event.name,
                description: event.description,
                poster_url: event.poster_url,
                location: event.location,
                starts_at: event.starts_at,
                ends_at: event.ends_at,
                capacity: event.capacity,
                registered_count: count,
                created_at: event.created_at,
            })
        });

        let summaries: Result<Vec<EventSummary>, AppError> =
            join_all(futures).await.into_iter().collect();

        let next_cursor = resp.last_evaluated_key().map(encode_cursor).transpose()?;

        Ok((summaries?, next_cursor))
    }

    async fn registered_count(&self, id: &EventId) -> Result<u32, AppError> {
        let resp = self
            .client
            .get_item()
            .table_name(&self.table_name)
            .key("pk", s(&event_pk(id)))
            .key("sk", s(event_count_sk()))
            .send()
            .await
            .map_err(|e| AppError::StorageError(e.to_string()))?;

        match resp.item {
            None => Ok(0),
            Some(item) => get_n_u32(&item, "count"),
        }
    }

    async fn save_poster(
        &self,
        event_id: &EventId,
        poster: &PosterSummary,
    ) -> Result<(), AppError> {
        self.client
            .put_item()
            .table_name(&self.table_name)
            .set_item(Some(poster_to_item(event_id, poster)))
            .send()
            .await
            .map_err(|e| AppError::StorageError(format!("failed to save poster: {e:?}")))?;

        Ok(())
    }

    async fn list_posters(&self, event_id: &EventId) -> Result<Vec<PosterSummary>, AppError> {
        let resp = self
            .client
            .query()
            .table_name(&self.table_name)
            .key_condition_expression("pk = :pk AND begins_with(sk, :prefix)")
            .expression_attribute_values(":pk", s(&event_pk(event_id)))
            .expression_attribute_values(":prefix", s(POSTER_SK_PREFIX))
            .consistent_read(true)
            .send()
            .await
            .map_err(|e| AppError::StorageError(format!("failed to list posters: {e:?}")))?;

        resp.items()
            .iter()
            .map(item_to_poster)
            .collect::<Result<Vec<_>, _>>()
    }

    async fn delete_poster(
        &self,
        event_id: &EventId,
        poster_id: &str,
    ) -> Result<Option<String>, AppError> {
        let resp = self
            .client
            .delete_item()
            .table_name(&self.table_name)
            .key("pk", s(&event_pk(event_id)))
            .key("sk", s(&poster_sk(poster_id)))
            .return_values(aws_sdk_dynamodb::types::ReturnValue::AllOld)
            .send()
            .await
            .map_err(|e| AppError::StorageError(format!("failed to delete poster: {e:?}")))?;

        match resp.attributes {
            Some(item) => get_s(&item, "object_key").map(Some),
            None => Ok(None),
        }
    }
}

fn event_to_item(event: &Event) -> HashMap<String, AttributeValue> {
    let mut item = HashMap::new();
    // FIX: s() expects &str, functions return String. Use &
    item.insert("pk".into(), s(&event_pk(&event.id)));
    item.insert("sk".into(), s(event_data_sk()));
    item.insert("event_id".into(), s(event.id.as_str()));
    item.insert("name".into(), s(&event.name));
    item.insert("location".into(), s(&event.location));
    item.insert("starts_at".into(), s(&event.starts_at.to_rfc3339()));
    item.insert("ends_at".into(), s(&event.ends_at.to_rfc3339()));
    item.insert("capacity".into(), n(event.capacity));
    item.insert("created_at".into(), s(&event.created_at.to_rfc3339()));

    if let Some(desc) = &event.description {
        item.insert("description".into(), s(desc));
    }
    if let Some(url) = &event.poster_url {
        item.insert("poster_url".into(), s(url));
    }
    item
}

fn poster_to_item(event_id: &EventId, poster: &PosterSummary) -> HashMap<String, AttributeValue> {
    let mut item = HashMap::new();
    item.insert("pk".into(), s(&event_pk(event_id)));
    item.insert("sk".into(), s(&poster_sk(&poster.id)));
    item.insert("poster_id".into(), s(&poster.id));
    item.insert("name".into(), s(&poster.name));
    item.insert("url".into(), s(&poster.url));
    item.insert("object_key".into(), s(&poster.object_key));
    item.insert("date_key".into(), s(&poster.date_key));
    item.insert("uploaded_at".into(), s(&poster.uploaded_at.to_rfc3339()));
    item
}

fn item_to_poster(item: &HashMap<String, AttributeValue>) -> Result<PosterSummary, AppError> {
    let uploaded_at = parse_timestamp(item, "uploaded_at")?;
    let date_key = get_s_opt(item, "date_key")?
        .unwrap_or_else(|| uploaded_at.format("%Y-%m-%d").to_string());

    Ok(PosterSummary {
        id: get_s(item, "poster_id")?,
        name: get_s(item, "name")?,
        url: get_s(item, "url")?,
        object_key: get_s(item, "object_key")?,
        date_key,
        uploaded_at,
    })
}

fn item_to_event(item: &HashMap<String, AttributeValue>) -> Result<Event, AppError> {
    Ok(Event::new(
        EventId::new(get_s(item, "event_id")?),
        get_s(item, "name")?.to_owned(),
        get_s_opt(item, "description")?,
        get_s_opt(item, "poster_url")?,
        get_s(item, "location")?.to_owned(),
        parse_timestamp(item, "starts_at")?,
        parse_timestamp(item, "ends_at")?,
        get_n_u32(item, "capacity")?,
        parse_timestamp(item, "created_at")?,
    ))
}

pub async fn increment_registered_count(
    client: &Client,
    table_name: &str,
    event_id: &EventId,
) -> Result<(), AppError> {
    client
        .update_item()
        .table_name(table_name)
        .key("pk", s(&event_pk(event_id)))
        .key("sk", s(&event_count_sk()))
        .update_expression("SET #c = #c + :one")
        .expression_attribute_names("#c", "count")
        .expression_attribute_values(":one", n(1u32))
        .send()
        .await
        .map_err(|e| AppError::StorageError(e.to_string()))?;
    Ok(())
}
