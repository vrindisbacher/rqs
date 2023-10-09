use queue::queue_service_server::{QueueService, QueueServiceServer}; 
use queue::{NewQueueRequest, NewQueueResponse};
use tonic::{Response, Status, Request};

mod queue;

#[derive(Debug, Default)]
pub struct Queue;

impl Queue {
    pub fn new_queue_server() -> QueueServiceServer<Queue> {
        QueueServiceServer::new(Queue::default())
    }
}

#[tonic::async_trait]
impl QueueService for Queue {
    async fn new_queue(&self, request: Request<NewQueueRequest>) -> Result<Response<NewQueueResponse>, Status> {
        todo!()
    }
}
