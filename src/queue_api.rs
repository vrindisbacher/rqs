use crate::app_types::{AppState, JsonResponse};
use actix_web::{web, HttpResponse};
use queue::Queue;
use request::NewQueueRequest;
use std::collections::hash_map::Entry;

pub(crate) mod queue;
mod request;

pub async fn new_queue(
    data: web::Data<AppState>,
    post_data: web::Json<NewQueueRequest>,
) -> HttpResponse {
    if post_data.max_batch <= 0 {
        return HttpResponse::BadRequest().json(JsonResponse::new(
            None::<String>,
            format!(
                "The max number of messages to send and receive at once {} is invalid",
                post_data.max_batch
            ),
        ));
    }
    if post_data.read_timeout <= 0 {
        return HttpResponse::BadRequest().json(JsonResponse::new(
            None::<String>,
            format!("The read timeout {} is invalid", post_data.read_timeout),
        ));
    }
    let queue = Queue::new(
        post_data.queue_id.to_owned(),
        post_data.read_timeout,
        post_data.max_batch,
    );
    let mut queues = data.get_queues().lock().await;
    let queue_uuid = &queue.get_uuid();
    match queues.entry(post_data.queue_id.to_owned()) {
        Entry::Vacant(_) => {
            queues.insert(post_data.queue_id.to_owned(), queue);
            HttpResponse::Accepted().json(JsonResponse::new(queue_uuid, None::<String>))
        }
        Entry::Occupied(_) => HttpResponse::Conflict().json(JsonResponse::new(
            None::<String>,
            format!("A queue with id {} already exists", post_data.queue_id),
        )),
    }
}

pub async fn list_queues(data: web::Data<AppState>) -> HttpResponse {
    let queues = data.get_queues().lock().await;
    let queue_uuids = queues.keys().collect::<Vec<&String>>();
    HttpResponse::Accepted().json(JsonResponse::new(queue_uuids, None::<String>))
}
