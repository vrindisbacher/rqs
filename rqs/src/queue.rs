use queue::queue_service_server::{QueueService, QueueServiceServer};
use queue::{NewQueueRequest, NewQueueResponse};
use tonic::{Request, Response, Status};

use crate::GLOBAL_DATA;

use self::queue::{DeleteQueueRequest, DeleteQueueResponse};

pub mod queue;

#[derive(Debug, Default)]
pub struct Queue;

impl Queue {
    pub fn new_queue_server() -> QueueServiceServer<Queue> {
        QueueServiceServer::new(Queue::default())
    }
}

#[tonic::async_trait]
impl QueueService for Queue {
    async fn new_queue(
        &self,
        request: Request<NewQueueRequest>,
    ) -> Result<Response<NewQueueResponse>, Status> {
        let inner = request.into_inner();
        let queue_id = inner.queue_id;
        let visibility_timeout = inner.visibility_timeout;
        let response = match GLOBAL_DATA
            .lock()
            .await
            .create_queue(queue_id, visibility_timeout)
            .await
        {
            Ok(_) => NewQueueResponse {
                success: true,
                data: "Successfully created queue".to_string(),
            },
            Err(e) => NewQueueResponse {
                success: false,
                data: format!("Failed to create queue. Failed with error: {e}"),
            },
        };
        Ok(Response::new(response))
    }

    async fn delete_queue(
        &self,
        request: Request<DeleteQueueRequest>,
    ) -> Result<Response<DeleteQueueResponse>, Status> {
        let queue_id = request.into_inner().queue_id;
        let response = match GLOBAL_DATA.lock().await.delete_queue(queue_id).await {
            Ok(_) => DeleteQueueResponse {
                success: true,
                data: "Successfully deleted queue".to_string(),
            },
            Err(e) => DeleteQueueResponse {
                success: true,
                data: format!("Failed to delete queue. Failed with error: {e}"),
            },
        };
        Ok(Response::new(response))
    }
}
