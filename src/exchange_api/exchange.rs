use actix_web::web;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::app_types::AppState;

#[derive(Debug)]
pub struct NoMatchingQueueError {
    id_failed_to_match: String,
}

impl NoMatchingQueueError {
    fn new(id_failed_to_match: String) -> Self {
        NoMatchingQueueError { id_failed_to_match }
    }

    pub fn to_string(&self) -> String {
        format!("Queue with id {} was not found", self.id_failed_to_match)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ExchangeType {
    FANOUT, // Fanout pushes message to all bound keys
    ID,     // Id pushes message to queues with particular id
}

pub struct Exchange {
    pub id: String,                  // the exchange id
    pub uuid: Uuid,                  // inner generated uuid for resource
    pub queue_ids: Vec<String>,      // the queues that are bound to the exchange
    pub exchange_type: ExchangeType, // what to do with messages
}

impl Exchange {
    pub fn new(id: String, queue_ids: Vec<String>, exchange_type: &ExchangeType) -> Self {
        Exchange {
            id,
            uuid: Uuid::new_v4(),
            queue_ids,
            exchange_type: match exchange_type {
                ExchangeType::FANOUT => ExchangeType::FANOUT,
                ExchangeType::ID => ExchangeType::ID,
            },
        }
    }

    pub async fn dispatch(
        &self,
        id: String,
        content: String,
        app_data: &web::Data<AppState>,
    ) -> Result<Vec<String>, NoMatchingQueueError> {
        match self.exchange_type {
            ExchangeType::ID => self.id_dispatch(id, content, app_data).await,
            ExchangeType::FANOUT => self.fanout_dispatch(id, content, app_data).await,
        }
    }

    async fn id_dispatch(
        &self,
        id: String,
        content: String,
        app_data: &web::Data<AppState>,
    ) -> Result<Vec<String>, NoMatchingQueueError> {
        let cipher = app_data.get_cipher().lock().await;
        for queue_id in self.queue_ids.iter() {
            // messages are sent to queues with same id as message id
            if *queue_id == id {
                let mut queues = app_data.get_queues().lock().await;
                let queue: &mut crate::queue_api::queue::Queue = match queues.get_mut(queue_id) {
                    None => {
                        return Err(NoMatchingQueueError::new(queue_id.to_owned()));
                    }
                    Some(q) => q,
                };
                let message = queue.add_to_queue(&cipher, id, content);
                return Ok(vec![message]);
            }
        }
        Err(NoMatchingQueueError::new(id))
    }

    async fn fanout_dispatch(
        &self,
        id: String,
        content: String,
        app_data: &web::Data<AppState>,
    ) -> Result<Vec<String>, NoMatchingQueueError> {
        let mut messages_produced = vec![];
        let cipher = app_data.get_cipher().lock().await;
        for queue_id in self.queue_ids.iter() {
            let mut queues = app_data.get_queues().lock().await;
            let queue = match queues.get_mut(queue_id) {
                None => {
                    return Err(NoMatchingQueueError::new(queue_id.to_owned()));
                }
                Some(q) => q,
            };
            let message = queue.add_to_queue(&cipher, id.to_owned(), content.to_owned());
            messages_produced.push(message);
        }
        Ok(messages_produced)
    }
}
