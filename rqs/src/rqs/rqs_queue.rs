use std::{io::Write, time::SystemTime, error::Error};
use serde::{Deserialize, Serialize};

use crate::rqs::{LOG_ROOT, QUEUE_LOG};

use super::rqs_utils::exponential_backoff;

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct MessageLogLine {
    pub id: u128,
    pub message_id: String,
    pub message_content: String,
    pub timestamp: u64,
    pub read_at: Option<u64>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct RQSQueue {
    name: String,
    messages: Vec<MessageLogLine>,
    visibility_timeout: u32,
    last_id: u128
}

impl RQSQueue {
    pub fn new(name: String, visibility_timeout: u32) -> Self {
        Self {
            name,
            messages: Vec::new(),
            visibility_timeout,
            last_id: 0
        }
    }

    pub fn get_name(&self) -> &String {
        &self.name
    }

    pub async fn revive_from_log(&mut self) {
        let log_file = match std::fs::read_to_string(format!(
                    "{LOG_ROOT}{QUEUE_LOG}{}/{}",
                    self.name, "message.log"
                )) {
            Ok(s) => s, 
            Err(_) => return,
        };
        for line in log_file.lines() {
            let message : MessageLogLine = serde_json::from_str(&String::from(line)).expect("Corrupt mesage log file");
            self.last_id = message.id;
            self.messages.push(message);
        }
    }

    async fn write_log(&self) {
        let mut file = exponential_backoff(|| 
             std::fs::OpenOptions::new()
                .create(true)
                .write(true)
                .open(format!(
                    "{LOG_ROOT}{QUEUE_LOG}{}/{}",
                    self.name, "message.log"
                ))
        ).await;
        for message in self.messages.iter() {
            file.write_fmt(format_args!(
                "{}\n",
                &serde_json::to_string(message).expect("Could not write log message to string")
            )).expect("Could not write to file")
        }
    }

    pub async fn add_message_to_queue(&mut self, message_id: String, message_content: String) {
        let now =
            exponential_backoff(|| SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)).await;
        let message = MessageLogLine {
            id: self.last_id + 1,
            message_id,
            message_content,
            timestamp: now.as_secs(),
            read_at: None,
        };
        self.messages.push(message);
        self.write_log().await;
        self.last_id += 1;
    }

    pub async fn take_message_from_queue(&mut self) -> Option<&MessageLogLine> {
        let now = exponential_backoff(|| SystemTime::now().duration_since(SystemTime::UNIX_EPOCH))
            .await
            .as_secs();
        let idx = self.messages.iter_mut().position(|x| { 
            if x.read_at.is_some() && x.read_at.unwrap() + self.visibility_timeout as u64 > now
            {
                x.read_at = Some(now);
                true
            } else {
                false
            }
        }); 
        match idx {
            None => None, 
            Some(idx) => {
                self.write_log().await;
                self.messages.get(idx)
            }
        }
    }
}
