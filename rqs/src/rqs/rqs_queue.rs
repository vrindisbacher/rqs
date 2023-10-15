use std::{io::Write, time::SystemTime};

use serde::{Deserialize, Serialize};

use crate::rqs::{LOG_ROOT, QUEUE_LOG};

use super::rqs_utils::exponential_backoff;

#[derive(Debug, Deserialize, Serialize)]
pub struct MessageLogLine<'a> {
    pub message_id: &'a str,
    pub message_content: &'a str,
    pub timestamp: u64,
    pub consumed: bool,
    pub read_at: Option<u64>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct RQSQueue {
    name: String,
    visibility_timeout: u32,
}

impl RQSQueue {
    pub fn new(name: String, visibility_timeout: u32) -> Self {
        Self {
            name,
            visibility_timeout,
        }
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
                    consumed: false,
                    timestamp: now.as_secs(),
                    read_at: None
                })?
            ))
        })
        .await;
    }

    pub async fn take_message_from_queue(&mut self) -> Option<String> {
        let log = exponential_backoff(|| {
            std::fs::read_to_string(format!(
                "{LOG_ROOT}{QUEUE_LOG}{}/{}",
                self.name, "message.log"
            ))
        })
        .await;
        let mut lines = log.lines().map(String::from);
        let now = exponential_backoff(|| SystemTime::now().duration_since(SystemTime::UNIX_EPOCH))
            .await
            .as_secs();
        let mut count = 0;
        lines.find(|x| {
            count += 1;
            let mut log_line: MessageLogLine = serde_json::from_str(x).expect("Log file corrupt");
            if !log_line.consumed
                || (log_line.read_at.is_some()
                    && log_line.read_at.unwrap() + self.visibility_timeout as u64 > now)
            {
                log_line.consumed = true; 
                log_line.read_at = Some(now);
                true
            } else {
                false
            }
        })
    }
}
