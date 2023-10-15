use message::message_service_server::{MessageService, MessageServiceServer};
use message::{
    AckMessageRequest, AckMessageResponse, ConsumeMessageRequest, ConsumeMessageResponse,
    NewMessageRequest, NewMessageResponse,
};
use tonic::{Request, Response, Status};

use crate::GLOBAL_DATA;

pub mod message;

#[derive(Debug, Default)]
pub struct Message;

impl Message {
    pub fn new_message_server() -> MessageServiceServer<Message> {
        MessageServiceServer::new(Message::default())
    }
}

#[tonic::async_trait]
impl MessageService for Message {
    async fn new_message(
        &self,
        request: Request<NewMessageRequest>,
    ) -> Result<Response<NewMessageResponse>, Status> {
        let message_request = request.into_inner();
        let mut queues = GLOBAL_DATA.lock().await;
        let handle_message_response = match queues
            .new_message(
                message_request.queue_id,
                message_request.message_id,
                message_request.message_content,
            )
            .await
        {
            Ok(_) => NewMessageResponse {
                data: "Successfully placed message on queue".to_string(),
                success: true,
            },
            Err(e) => NewMessageResponse {
                data: format!("Failed to place message with error {e}"),
                success: false,
            },
        };
        Ok(Response::new(handle_message_response))
    }

    async fn consume_message(
        &self,
        request: Request<ConsumeMessageRequest>,
    ) -> Result<Response<ConsumeMessageResponse>, Status> {
        let queue_id = request.into_inner().queue_id;
        let mut queues = GLOBAL_DATA.lock().await;
        let res = queues.get_message(queue_id).await;
        let message_response = match res {
            Ok(message) => match message {
                Some(message) => ConsumeMessageResponse {
                    id: Some(message.id),
                    message_id: Some(message.message_id.clone()),
                    message_content: Some(message.message_content.clone()),
                    error_message: None,
                    success: true,
                },
                None => ConsumeMessageResponse {
                    id: None,
                    message_id: None,
                    message_content: None,
                    error_message: None,
                    success: true,
                },
            },
            Err(e) => ConsumeMessageResponse {
                id: None,
                message_id: None,
                message_content: None,
                error_message: Some(format!("Failed to consume message. Error: {e}")),
                success: false,
            },
        };
        Ok(Response::new(message_response))
    }

    async fn ack_message(
        &self,
        request: Request<AckMessageRequest>,
    ) -> Result<Response<AckMessageResponse>, Status> {
        let inner = request.into_inner();
        let queue_id = inner.queue_id;
        let id = inner.id;
        let mut queues = GLOBAL_DATA.lock().await;
        let ack_response = match queues.ack_message(queue_id, id).await {
            Ok(_) => AckMessageResponse {
                data: format!("Successfully ack'd message with id {id}"),
                success: true,
            },
            Err(e) => AckMessageResponse {
                data: format!("Failed to ack message. Failed with {e}"),
                success: false,
            },
        };
        Ok(Response::new(ack_response))
    }
}
