use std::{io::Write, time::SystemTime};

use serde::{Deserialize, Serialize};

use crate::rqs::{LOG_ROOT, QUEUE_LOG};

use super::rqs_utils::exponential_backoff;

#[derive(Debug, Deserialize, Serialize)]
pub struct MessageLogLine<'a> {
    pub message_id: &'a str,
    pub message_content: &'a str,
    pub timestamp: u64,
}

#[derive(Debug, PartialEq, Eq)]
pub struct RQSQueue {
    name: String,
}

impl RQSQueue {
    pub fn new(name: String) -> Self {
        Self { name }
    }

    pub fn get_name(&self) -> &String {
        &self.name
    }

    pub async fn add_message_to_queue(&self, message_id: &String, message_content: &String) {
        let now =
            exponential_backoff(|| SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)).await;
        exponential_backoff(|| {
            let mut file = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(format!(
                    "{LOG_ROOT}{QUEUE_LOG}{}/{}",
                    self.name, "message.log"
                ))?;
            file.write_fmt(format_args!(
                "{}\n",
                &serde_json::to_string(&MessageLogLine {
                    message_id,
                    message_content,
                    timestamp: now.as_secs(),
                })?
            ))
        })
        .await;
    }

    pub fn take_message_from_queue(&mut self) {}
}
