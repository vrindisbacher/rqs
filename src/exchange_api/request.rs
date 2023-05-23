use serde::{Deserialize, Serialize};

use super::exchange::ExchangeType;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewExchangeRequest {
    pub id: String,
    pub queue_ids: Vec<String>,
    #[serde(alias = "name")]
    pub exchange_type: ExchangeType,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExchangeEntry {
    pub id: String,
    pub queue_ids: Vec<String>,
    #[serde(alias = "name")]
    pub exchange_type: ExchangeType,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewMessage {
    pub message_id: String,
    pub content: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewMessageRequest {
    pub messages: Vec<NewMessage>,
    pub exchange_id: String,
}
