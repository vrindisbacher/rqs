use message::message_service_server::{MessageServiceServer, MessageService};
use message::{NewMessageRequest, NewMessageResponse, ConsumeMessageRequest, ConsumeMessageResponse, AckMessageRequest, AckMessageResponse};
use tonic::{Request, Response, Status};

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

    async fn new_message(&self, request: Request<NewMessageRequest>) -> Result<Response<NewMessageResponse>, Status> {
        todo!()
    }

    async fn consume_message(&self, request: Request<ConsumeMessageRequest>) -> Result<Response<ConsumeMessageResponse>, Status> {
        todo!()
    }

    async fn ack_message(&self, request: Request<AckMessageRequest>) -> Result<Response<AckMessageResponse>, Status> {
        todo!()
    }
}
