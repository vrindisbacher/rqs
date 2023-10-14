use std::{io::Write, time::SystemTime};

use rqs_queue::RQSQueue;
use rqs_types::{RQSError, RQSEvent};
use rqs_utils::exponential_backoff;

mod rqs_queue;
pub mod rqs_types;
mod rqs_utils;

pub static LOG_ROOT: &'static str = "tmp/log/";
pub static EVENT_LOG: &'static str = "event.log";
pub static QUEUE_LOG: &'static str = "queues/";

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

    pub async fn handle_event(&mut self, event: RQSEvent) -> Result<(), RQSError> {
        match event.clone() {
            RQSEvent::QueueCreated { queue_id } => {
                self.create_queue(queue_id).await?;
                self.log_event(event).await;
            }
            RQSEvent::QueueDeleted { queue_id } => {
                self.delete_queue(queue_id).await?;
                self.log_event(event).await;
            }
            RQSEvent::NewMessage {
                queue_id,
                message_id,
                message_content,
            } => {
                let queue = match self.queues.iter().find(|x| x.get_name() == &queue_id) {
                    None => {
                        return Err(RQSError::FailedToAddMessage(format!(
                            "Queue {queue_id} does not exist"
                        )))
                    }
                    Some(queue) => queue,
                };
                queue
                    .add_message_to_queue(&message_id, &message_content)
                    .await;
            }
        }
        Ok(())
    }

    pub async fn revive_from_log(&mut self) {
        if std::fs::metadata(format!("{LOG_ROOT}{EVENT_LOG}")).is_err() {
            return;
        }
        let event_log =
            exponential_backoff(|| std::fs::read_to_string(format!("{LOG_ROOT}{EVENT_LOG}"))).await;

        for line in event_log.lines().map(String::from).collect::<Vec<String>>() {
            let parts = line.split(" ").collect::<Vec<&str>>();
            if !parts.len() == 3 {
                panic!("Log file at path {LOG_ROOT}{EVENT_LOG} is corrupt. The failing line was {line}")
            }
            let instr = parts.get(0).expect(&format!(
                "Log file at path {LOG_ROOT}{EVENT_LOG} is corrupt. The failing line was {line}"
            ));
            let name = parts.get(1).expect(&format!("Log file at path {LOG_ROOT}{EVENT_LOG} is corrupt. The failing line was {line}")).split(":").collect::<Vec<&str>>().get(1).expect(&format!("Log file at path {LOG_ROOT}{EVENT_LOG} is corrupt. The failing line was {line}")).to_string();
            let _timestamp = parts.get(2).expect(&format!("Log file at path {LOG_ROOT}{EVENT_LOG} is corrupt. The failing line was {line}")).split(":").collect::<Vec<&str>>().get(1).expect(&format!("Log file at path {LOG_ROOT}{EVENT_LOG} is corrupt. The failing line was {line}"));
            match *instr {
                "QueueCreated" => self.create_queue(name).await.expect("Could not create queue"),
                "QueueDeleted" => self.delete_queue(name).await.expect("Could not delete queue"),
                _ => panic!("Log file at path {LOG_ROOT}{EVENT_LOG} is corrupt. The failing line was {line}")
            };
        }
    }

    async fn log_event(&self, event: RQSEvent) {
        let now =
            exponential_backoff(|| SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)).await;
        match event {
            RQSEvent::QueueCreated { queue_id: name } => {
                exponential_backoff(|| {
                    let mut file = std::fs::OpenOptions::new()
                        .create(true)
                        .append(true)
                        .open(format!("{LOG_ROOT}{EVENT_LOG}"))?;
                    file.write_fmt(format_args!(
                        "QueueCreated name:{name} timestamp:{}\n",
                        now.as_secs()
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
                        "QueueDeleted name:{name} timestamp:{}\n",
                        now.as_secs()
                    ))
                })
                .await;
            }
            RQSEvent::NewMessage {
                queue_id: _,
                message_id: _,
                message_content: _,
            } => (),
        }
    }

    async fn create_queue(&mut self, name: String) -> Result<(), RQSError> {
        if self.queues.iter().any(|x| x.get_name() == &name) {
            return Err(RQSError::FailedToCreateQueue(format!(
                "Queue with name {name} already exists"
            )));
        }
        exponential_backoff(|| std::fs::create_dir_all(format!("{LOG_ROOT}{QUEUE_LOG}{name}")))
            .await;
        self.queues.push(RQSQueue::new(name));
        Ok(())
    }

    async fn delete_queue(&mut self, name: String) -> Result<(), RQSError> {
        if !self.queues.iter().any(|x| x.get_name() == &name) {
            return Err(RQSError::FailedToDeleteQueue(format!(
                "Queue with name {name} does not exist"
            )));
        }
        exponential_backoff(|| std::fs::remove_dir_all(format!("{LOG_ROOT}{QUEUE_LOG}{name}")))
            .await;
        self.queues.retain(|x| x.get_name() != &name);
        Ok(())
    }
}

#[cfg(test)]
mod rqs_test {
    use serial_test::serial;

    use super::{rqs_types::RQSEvent, EVENT_LOG, LOG_ROOT, RQS};

    // NOTE: using serial because these tests cannot be run concurrently

    fn delete_event_log() {
        std::fs::remove_file(format!("{LOG_ROOT}{EVENT_LOG}"));
    }

    #[tokio::test]
    #[serial]
    async fn test_revive() {
        delete_event_log();
        let mut rqs = RQS::new();
        rqs.revive_from_log().await;
        rqs.handle_event(RQSEvent::QueueCreated {
            queue_id: "queue_1".to_string(),
        })
        .await
        .expect("Could not create queue_1");
        rqs.handle_event(RQSEvent::QueueCreated {
            queue_id: "queue_2".to_string(),
        })
        .await
        .expect("Could not create queue_2");
        rqs.handle_event(RQSEvent::QueueDeleted {
            queue_id: "queue_1".to_string(),
        })
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
        rqs.handle_event(RQSEvent::QueueCreated {
            queue_id: "queue_1".to_string(),
        })
        .await
        .expect("Could not create queue_1");
        rqs.handle_event(RQSEvent::QueueCreated {
            queue_id: "queue_2".to_string(),
        })
        .await
        .expect("Could not create queue_2");
        rqs.handle_event(RQSEvent::QueueDeleted {
            queue_id: "queue_1".to_string(),
        })
        .await
        .expect("Could not delete queue_1");

        let mut rqs_from_revive = RQS::new();
        rqs_from_revive.revive_from_log().await;
        rqs_from_revive
            .handle_event(RQSEvent::QueueCreated {
                queue_id: "queue_3".to_string(),
            })
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
        rqs.handle_event(RQSEvent::QueueCreated {
            queue_id: "queue_1".to_string(),
        })
        .await
        .expect("Could not create queue_1");
        assert!(rqs
            .handle_event(RQSEvent::QueueCreated {
                queue_id: "queue_1".to_string()
            })
            .await
            .is_err());
    }

    #[tokio::test]
    #[serial]
    async fn test_queue_to_delete_does_not_exist() {
        delete_event_log();
        let mut rqs = RQS::new();
        rqs.revive_from_log().await;
        rqs.handle_event(RQSEvent::QueueCreated {
            queue_id: "queue_1".to_string(),
        })
        .await
        .expect("Could not create queue_1");
        assert!(rqs
            .handle_event(RQSEvent::QueueDeleted {
                queue_id: "queue_2".to_string()
            })
            .await
            .is_err());
    }

    #[tokio::test]
    #[serial]
    async fn test_create_message() {
        delete_event_log();
        let mut rqs = RQS::new();
        rqs.revive_from_log().await;
        rqs.handle_event(RQSEvent::QueueCreated {
            queue_id: "queue_1".to_string(),
        })
        .await
        .expect("Could not create queue_1");
        rqs.handle_event(RQSEvent::NewMessage { 
            message_id: "hello".to_string(), 
            message_content: "{helloworlditsme: 1}".to_string(),
            queue_id: "queue_1".to_string()
        }).await.expect("Failed to send message");
    }
}
