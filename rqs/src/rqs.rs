use std::{io::Write, time::SystemTime};

use rqs_queue::RQSQueue;
use rqs_types::{RQSError, RQSEvent};
use rqs_utils::exponential_backoff;
use serde::{Deserialize, Serialize};

use self::rqs_queue::MessageLogLine;

mod rqs_queue;
pub mod rqs_types;
mod rqs_utils;

pub static LOG_ROOT: &'static str = "tmp/log/";
pub static EVENT_LOG: &'static str = "event.log";
pub static QUEUE_LOG: &'static str = "queues/";

#[derive(Serialize, Deserialize)]
pub struct RQSLogLine<'a> {
    pub event: &'a str,
    pub visibility_timeout: u32,
    pub queue_id: &'a str,
    pub timestamp: u64,
}

#[derive(Debug, PartialEq)]
pub struct RQS {
    queues: Vec<RQSQueue>,
}

impl RQS {
    pub fn new() -> Self {
        Self { queues: Vec::new() }
    }

    pub fn get_queues(&self) -> &Vec<RQSQueue> {
        &self.queues
    }

    pub fn clear(&mut self) {
        self.queues.clear();
    }

    pub async fn revive_from_log(&mut self) {
        if std::fs::metadata(format!("{LOG_ROOT}{EVENT_LOG}")).is_err() {
            return;
        }
        let event_log =
            exponential_backoff(|| std::fs::read_to_string(format!("{LOG_ROOT}{EVENT_LOG}"))).await;

        for line in event_log.lines().map(String::from).collect::<Vec<String>>() {
            let event: RQSLogLine = serde_json::from_str(&line).expect(&format!(
                "Log file at path {LOG_ROOT}{EVENT_LOG} is corrupt. The failing line was {line}"
            ));
            match event.event {
                "QueueCreated" => self.create_queue(event.queue_id.to_string(), event.visibility_timeout).await.expect("Could not create queue"), 
                "QueueDeleted" => self.delete_queue(event.queue_id.to_string()).await.expect("Could not delete queue"),
                _ => panic!("Log file at path {LOG_ROOT}{EVENT_LOG} is corrupt. The failing line was {line}")
            };
        }

        for queue in self.queues.iter_mut() {
            queue.revive_from_log().await;
        }
    }

    async fn log_event(&self, event: RQSEvent) {
        let now =
            exponential_backoff(|| SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)).await;
        match event {
            RQSEvent::QueueCreated {
                queue_id: name,
                visibility_timeout,
            } => {
                exponential_backoff(|| {
                    let mut file = std::fs::OpenOptions::new()
                        .create(true)
                        .append(true)
                        .open(format!("{LOG_ROOT}{EVENT_LOG}"))?;
                    file.write_fmt(format_args!(
                        "{}\n",
                        &serde_json::to_string(&RQSLogLine {
                            event: "QueueCreated",
                            visibility_timeout,
                            queue_id: name.as_str(),
                            timestamp: now.as_secs(),
                        })?
                    ))
                })
                .await;
            }
            RQSEvent::QueueDeleted { queue_id: name } => {
                exponential_backoff(|| {
                    let mut file = std::fs::OpenOptions::new()
                        .create(true)
                        .append(true)
                        .open(format!("{LOG_ROOT}{EVENT_LOG}"))?;
                    file.write_fmt(format_args!(
                        "{}\n",
                        &serde_json::to_string(&RQSLogLine {
                            event: "QueueDeleted",
                            visibility_timeout: 0,
                            queue_id: name.as_str(),
                            timestamp: now.as_secs(),
                        })?
                    ))
                })
                .await;
            }
        }
    }

    pub async fn create_queue(
        &mut self,
        queue_id: String,
        visibility_timeout: u32,
    ) -> Result<(), RQSError> {
        if self.queues.iter().any(|x| x.get_name() == &queue_id) {
            return Err(RQSError::FailedToCreateQueue(format!(
                "Queue with name {queue_id} already exists"
            )));
        }
        exponential_backoff(|| std::fs::create_dir_all(format!("{LOG_ROOT}{QUEUE_LOG}{queue_id}")))
            .await;
        self.queues
            .push(RQSQueue::new(queue_id.clone(), visibility_timeout));
        self.log_event(RQSEvent::QueueCreated {
            queue_id,
            visibility_timeout,
        })
        .await;
        Ok(())
    }

    pub async fn delete_queue(&mut self, queue_id: String) -> Result<(), RQSError> {
        if !self.queues.iter().any(|x| x.get_name() == &queue_id) {
            return Err(RQSError::FailedToDeleteQueue(format!(
                "Queue with name {queue_id} does not exist"
            )));
        }
        exponential_backoff(|| std::fs::remove_dir_all(format!("{LOG_ROOT}{QUEUE_LOG}{queue_id}")))
            .await;
        self.queues.retain(|x| x.get_name() != &queue_id);
        self.log_event(RQSEvent::QueueDeleted { queue_id }).await;
        Ok(())
    }

    pub async fn new_message(
        &mut self,
        queue_id: String,
        message_id: String,
        message_content: String,
    ) -> Result<(), RQSError> {
        let queue = match self.queues.iter_mut().find(|x| x.get_name() == &queue_id) {
            Some(q) => q,
            None => {
                return Err(RQSError::FailedToAddMessage(format!(
                    "Queue with name {queue_id} does not exist"
                )))
            }
        };
        queue
            .add_message_to_queue(message_id, message_content)
            .await;
        Ok(())
    }

    pub async fn get_message(
        &mut self,
        queue_id: String,
    ) -> Result<Option<&MessageLogLine>, RQSError> {
        let queue = match self.queues.iter_mut().find(|x| x.get_name() == &queue_id) {
            Some(q) => q,
            None => {
                return Err(RQSError::FailedToGetMessage(format!(
                    "Queue with name {queue_id} does not exist"
                )))
            }
        };
        Ok(queue.take_message_from_queue().await)
    }
}

#[cfg(test)]
mod rqs_test {
    use serial_test::serial;

    use crate::rqs::rqs_queue::MessageLogLine;

    use super::{LOG_ROOT, QUEUE_LOG, RQS};

    // NOTE: using serial because these tests cannot be run concurrently

    fn delete_event_log() {
        let _ = std::fs::remove_dir_all(format!("{LOG_ROOT}"));
    }

    #[tokio::test]
    #[serial]
    async fn test_revive() {
        delete_event_log();
        let mut rqs = RQS::new();
        rqs.revive_from_log().await;
        rqs.create_queue("queue_1".to_string(), 10)
            .await
            .expect("Could not create queue_1");
        rqs.create_queue("queue_2".to_string(), 10)
            .await
            .expect("Could not create queue_2");
        rqs.delete_queue("queue_1".to_string())
            .await
            .expect("Could not delete queue_1");

        let mut rqs_from_revive = RQS::new();
        rqs_from_revive.revive_from_log().await;
        assert_eq!(rqs, rqs_from_revive);
    }

    #[tokio::test]
    #[serial]
    async fn test_revive_and_add() {
        delete_event_log();
        let mut rqs = RQS::new();
        rqs.revive_from_log().await;
        rqs.create_queue("queue_1".to_string(), 10)
            .await
            .expect("Could not create queue_1");
        rqs.create_queue("queue_2".to_string(), 10)
            .await
            .expect("Could not create queue_2");
        rqs.delete_queue("queue_1".to_string())
            .await
            .expect("Could not delete queue_1");

        let mut rqs_from_revive = RQS::new();
        rqs_from_revive.revive_from_log().await;
        rqs_from_revive
            .create_queue("queue_3".to_string(), 10)
            .await
            .expect("Could not create queue_3");
        let names = rqs_from_revive
            .queues
            .iter()
            .map(|x| x.get_name())
            .collect::<Vec<&String>>();
        assert_eq!(names, vec!["queue_2", "queue_3"]);
    }

    #[tokio::test]
    #[serial]
    async fn test_queue_already_exists() {
        delete_event_log();
        let mut rqs = RQS::new();
        rqs.revive_from_log().await;
        rqs.create_queue("queue_1".to_string(), 10)
            .await
            .expect("Could not create queue_1");
        assert!(rqs.create_queue("queue_1".to_string(), 10).await.is_err());
    }

    #[tokio::test]
    #[serial]
    async fn test_queue_to_delete_does_not_exist() {
        delete_event_log();
        let mut rqs = RQS::new();
        rqs.revive_from_log().await;
        rqs.create_queue("queue_1".to_string(), 10)
            .await
            .expect("Could not create queue_1");
        assert!(rqs.delete_queue("queue_2".to_string(),).await.is_err());
    }

    #[tokio::test]
    #[serial]
    async fn test_create_message() {
        delete_event_log();
        let mut rqs = RQS::new();
        rqs.revive_from_log().await;
        rqs.create_queue("queue_1".to_string(), 10)
            .await
            .expect("Could not create queue_1");
        rqs.new_message(
            "queue_1".to_string(),
            "hello".to_string(),
            "{helloworlditsme: 1}".to_string(),
        )
        .await
        .expect("Failed to send message");

        let event_log = std::fs::read_to_string(format!(
            "{LOG_ROOT}{QUEUE_LOG}{}/{}",
            "queue_1", "message.log"
        ))
        .unwrap();
        let lines = event_log.lines().map(String::from).collect::<Vec<String>>();
        assert_eq!(lines.len(), 1);
        let line = &lines[0];
        let message: MessageLogLine = serde_json::from_str(&line).unwrap();
        assert_eq!(message.message_id, "hello");
        assert_eq!(message.message_content, "{helloworlditsme: 1}");
    }

    #[tokio::test]
    #[serial]
    async fn test_create_and_consume_message() {
        delete_event_log();
        let mut rqs = RQS::new();
        rqs.revive_from_log().await;
        rqs.create_queue("queue_1".to_string(), 10)
            .await
            .expect("Could not create queue_1");
        rqs.new_message(
            "queue_1".to_string(),
            "hello".to_string(),
            "{helloworlditsme: 1}".to_string(),
        )
        .await
        .expect("Failed to send message");

        let message = rqs
            .get_message("queue_1".to_string())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(message.message_id, "hello");
        assert_eq!(message.message_content, "{helloworlditsme: 1}");
        assert_eq!(message.id, 1);
    }

    #[tokio::test]
    #[serial]
    async fn test_message_visibility_timeout() {
        delete_event_log();
        let mut rqs = RQS::new();
        rqs.revive_from_log().await;
        rqs.create_queue("queue_1".to_string(), 10)
            .await
            .expect("Could not create queue_1");
        rqs.new_message(
            "queue_1".to_string(),
            "hello".to_string(),
            "{helloworlditsme: 1}".to_string(),
        )
        .await
        .expect("Failed to send message");

        let message = rqs
            .get_message("queue_1".to_string())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(message.message_id, "hello");
        assert_eq!(message.message_content, "{helloworlditsme: 1}");
        assert_eq!(message.id, 1);

        let message = rqs.get_message("queue_1".to_string()).await.unwrap();
        assert!(message.is_none());
    }
}
