use serde::Deserialize;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewQueueRequest {
    pub read_timeout: u32,
    pub queue_id: String,
}
