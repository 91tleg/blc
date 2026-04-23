use async_trait::async_trait;
use aws_sdk_dynamodb::{
    types::{AttributeValue, Put, TransactWriteItem, Update},
    Client,
};
use std::collections::HashMap;

use crate::application::{errors::AppError, ports::RegistrationsRepo};
use crate::domain::{
    registration::Registration,
    types::{EventId, RegistrationId},
};
use crate::infrastructure::dynamo::{
    decode_cursor,
    encode_cursor,
    event_count_sk,
    get_s,
    n, // Added n
    parse_timestamp,
    registration_email_gsi_sk,
    registration_pk,
    registration_sk,
    s,
};

pub struct DynamoRegistrationsRepo {
    pub client: Client,
    pub table_name: String,
    pub email_gsi_name: String,
    pub events_table_name: String,
}

impl DynamoRegistrationsRepo {
    pub fn new(
        client: Client,
        table_name: impl Into<String>,
        email_gsi_name: impl Into<String>,
        events_table_name: impl Into<String>,
    ) -> Self {
        Self {
            client,
            table_name: table_name.into(),
            email_gsi_name: email_gsi_name.into(),
            events_table_name: events_table_name.into(),
        }
    }
}

fn registration_to_item(reg: &Registration) -> HashMap<String, AttributeValue> {
    let mut item = HashMap::new();
    // FIX: Borrow the results of the generator functions
    item.insert("pk".into(), s(&registration_pk(&reg.event_id)));
    item.insert("sk".into(), s(&registration_sk(&reg.id)));
    item.insert("gsi_sk".into(), s(&registration_email_gsi_sk(&reg.email)));
    item.insert("registration_id".into(), s(reg.id.as_str()));
    item.insert("event_id".into(), s(reg.event_id.as_str()));
    item.insert("full_name".into(), s(&reg.full_name));
    item.insert("email".into(), s(&reg.email));

    if let Some(phone) = &reg.phone_number {
        item.insert("phone_number".into(), s(phone));
    }

    item.insert("registered_at".into(), s(&reg.registered_at.to_rfc3339()));
    item
}

fn item_to_registration(item: &HashMap<String, AttributeValue>) -> Result<Registration, AppError> {
    let phone_number = item.get("phone_number").and_then(|av| match av {
        AttributeValue::S(s) => Some(s.clone()),
        _ => None,
    });

    Ok(Registration::new(
        RegistrationId::new(get_s(item, "registration_id")?),
        EventId::new(get_s(item, "event_id")?),
        get_s(item, "full_name")?.to_owned(),
        get_s(item, "email")?.to_owned(),
        phone_number,
        parse_timestamp(item, "registered_at")?,
    ))
}

#[async_trait]
impl RegistrationsRepo for DynamoRegistrationsRepo {
    async fn save(&self, registration: &Registration) -> Result<(), AppError> {
        let reg_put = Put::builder()
            .table_name(&self.table_name)
            .set_item(Some(registration_to_item(registration)))
            .condition_expression("attribute_not_exists(pk) AND attribute_not_exists(sk)")
            .build()
            .map_err(|e| AppError::StorageError(e.to_string()))?;

        let count_update = Update::builder()
            .table_name(&self.events_table_name)
            .key(
                "pk",
                s(&format!("EVENT#{}", registration.event_id.as_str())),
            )
            .key("sk", s(event_count_sk()))
            .update_expression("SET #c = #c + :one")
            .expression_attribute_names("#c", "count")
            .expression_attribute_values(":one", n(1u32))
            .build()
            .map_err(|e| AppError::StorageError(e.to_string()))?;

        self.client
            .transact_write_items()
            .transact_items(TransactWriteItem::builder().put(reg_put).build())
            .transact_items(TransactWriteItem::builder().update(count_update).build())
            .send()
            .await
            .map_err(|e| {
                if e.to_string().contains("TransactionCanceledException") {
                    AppError::AlreadyRegistered
                } else {
                    AppError::StorageError(format!("Transaction failed: {e:?}"))
                }
            })?;

        Ok(())
    }

    async fn find_by_event_and_email(
        &self,
        event_id: &EventId,
        email: &str,
    ) -> Result<Option<Registration>, AppError> {
        let resp = self
            .client
            .query()
            .table_name(&self.table_name)
            .index_name(&self.email_gsi_name)
            .key_condition_expression("pk = :pk AND gsi_sk = :gsi_sk")
            .expression_attribute_values(":pk", s(&registration_pk(event_id)))
            .expression_attribute_values(":gsi_sk", s(&registration_email_gsi_sk(email)))
            .limit(1)
            .send()
            .await
            .map_err(|e| AppError::StorageError(e.to_string()))?;

        match resp.items().first() {
            None => Ok(None),
            Some(item) => item_to_registration(item).map(Some),
        }
    }

    async fn list_by_event(
        &self,
        event_id: &EventId,
        limit: u32,
        cursor: Option<String>,
    ) -> Result<(Vec<Registration>, Option<String>, u32), AppError> {
        let mut req = self
            .client
            .query()
            .table_name(&self.table_name)
            .key_condition_expression("pk = :pk AND begins_with(sk, :prefix)")
            .expression_attribute_values(":pk", s(&registration_pk(event_id)))
            .expression_attribute_values(":prefix", s("REG#"))
            .consistent_read(true)
            .limit(limit as i32);

        if let Some(c) = cursor {
            req = req.set_exclusive_start_key(Some(decode_cursor(&c)?));
        }

        let resp = req
            .send()
            .await
            .map_err(|e| AppError::StorageError(e.to_string()))?;

        let registrations = resp
            .items()
            .iter()
            .map(item_to_registration)
            .collect::<Result<Vec<_>, _>>()?;

        let next_cursor = resp.last_evaluated_key().map(encode_cursor).transpose()?;

        let page_count = resp.count() as u32;

        Ok((registrations, next_cursor, page_count))
    }

    async fn find_by_id(
        &self,
        registration_id: &RegistrationId,
    ) -> Result<Option<Registration>, AppError> {
        let resp = self
            .client
            .query()
            .table_name(&self.table_name)
            .index_name("registration_id-index")
            .key_condition_expression("registration_id = :rid")
            .expression_attribute_values(":rid", s(registration_id.as_str()))
            .send()
            .await
            .map_err(|e| AppError::StorageError(e.to_string()))?;

        match resp.items().first() {
            None => Ok(None),
            Some(item) => item_to_registration(item).map(Some),
        }
    }
}
