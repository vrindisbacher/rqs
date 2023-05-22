use serde::{Deserialize, Serialize};

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
    pub queue_id: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetMessageRequest {
    pub queue_id: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetMessageResponse {
    pub message_id: String,
    pub content: String,
    pub uuid: String,
}

impl GetMessageResponse {
    pub fn new(message_id: String, content: String, uuid: String) -> Self {
        GetMessageResponse {
            message_id,
            content,
            uuid,
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteMessageRequest {
    pub queue_id: String,
    pub message_uuid: String,
}
