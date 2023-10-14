use message::message_service_server::{MessageService, MessageServiceServer};
use message::{
    AckMessageRequest, AckMessageResponse, ConsumeMessageRequest, ConsumeMessageResponse,
    NewMessageRequest, NewMessageResponse,
};
use tonic::{Request, Response, Status};

use crate::rqs::rqs_types::RQSEvent;
use crate::GLOBAL_DATA;

mod message;

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
            .handle_event(RQSEvent::NewMessage {
                queue_id: message_request.queue_id,
                message_id: message_request.message_id,
                message_content: message_request.message_content,
            })
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
        todo!()
    }

    async fn ack_message(
        &self,
        request: Request<AckMessageRequest>,
    ) -> Result<Response<AckMessageResponse>, Status> {
        todo!()
    }
}
