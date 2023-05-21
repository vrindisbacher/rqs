use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewMessageRequest {
    pub queue_id: String,
    pub message_id: String,
    pub content: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetMessageRequest {
    pub queue_id: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetMessageResponse {
    message_id: String,
    content: String,
    uuid: String,
}

impl GetMessageResponse {
    pub fn new(message_id: String, content: String, uuid: String) -> GetMessageResponse {
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
