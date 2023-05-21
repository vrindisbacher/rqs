use crate::app_types::{AppState, JsonResponse};
use actix_web::{web, HttpResponse};
use queue::Queue;
use request::NewQueueRequest;

pub(crate) mod queue;
mod request;

pub async fn new_queue(
    data: web::Data<AppState>,
    post_data: web::Json<NewQueueRequest>,
) -> HttpResponse {
    let queue = Queue::new(post_data.queue_id.to_owned(), post_data.read_timeout);
    let mut queues = data.get_queues().lock().await;
    let queue_uuid = &queue.get_uuid();
    queues.push(queue);
    HttpResponse::Accepted().json(JsonResponse::new(queue_uuid, None::<String>))
}

pub async fn list_queues(data: web::Data<AppState>) -> HttpResponse {
    let queues = data.get_queues().lock().await;
    let queue_uuids = queues.iter().map(|q| q.get_id()).collect::<Vec<String>>();
    HttpResponse::Accepted().json(JsonResponse::new(queue_uuids, None::<String>))
}
