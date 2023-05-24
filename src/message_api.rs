use crate::app_types::{AppState, JsonResponse};
use actix_web::{web, HttpResponse};
use request::{DeleteMessageRequest, GetMessageRequest, GetMessageResponse, NewMessageRequest};

mod request;

pub async fn add_message_to_queue(
    data: web::Data<AppState>,
    post_data: web::Json<NewMessageRequest>,
) -> HttpResponse {
    let queue_id = &post_data.queue_id;

    let mut queues = data.get_queues().lock().await;
    let queue = match queues.get_mut(queue_id) {
        None => {
            return HttpResponse::BadRequest().json(JsonResponse::new(
                None::<String>,
                format!("No queue with id {} was found", queue_id),
            ))
        }
        Some(q) => q,
    };

    let cipher = data.get_cipher().lock().await;

    let messages_to_add = &post_data.messages;
    let mut messages_to_send = vec![];
    for message in messages_to_add.iter() {
        let id = message.message_id.to_owned();
        let content = message.content.to_owned();
        let message_added = queue.add_to_queue(&cipher, id, content);
        messages_to_send.push(message_added);
    }

    HttpResponse::Accepted().json(JsonResponse::new(messages_to_send, None::<String>))
}

pub async fn delete_message(
    data: web::Data<AppState>,
    post_data: web::Json<DeleteMessageRequest>,
) -> HttpResponse {
    let queue_id = &post_data.queue_id;
    let mut queues = data.get_queues().lock().await;
    let queue = match queues.get_mut(queue_id) {
        None => {
            return HttpResponse::BadRequest().json(JsonResponse::new(
                None::<String>,
                format!("No queue with id {} was found", queue_id),
            ))
        }
        Some(q) => q,
    };

    let message_uuid = &post_data.message_uuid;
    match queue.rem_from_queue(message_uuid) {
        None => HttpResponse::BadRequest().json(JsonResponse::new(
            None::<String>,
            format!("No message with uuid {} found, or the message is past the set read timeout - you cannot delete a message past its read timeout because another consumer may be using it.", message_uuid),
        )),
        Some(_) => HttpResponse::Accepted().json(JsonResponse::new(
            format!("Successfully deleted uuid {}", message_uuid),
            None::<String>,
        )),
    }
}

pub async fn get_message(
    data: web::Data<AppState>,
    query_data: web::Query<GetMessageRequest>,
) -> HttpResponse {
    let queue_id = &query_data.queue_id;

    let mut queues = data.get_queues().lock().await;
    let queue = match queues.get_mut(queue_id) {
        None => {
            return HttpResponse::BadRequest().json(JsonResponse::new(
                None::<String>,
                format!("No queue with id {} was found", queue_id),
            ))
        }
        Some(q) => q,
    };

    let cipher = data.get_cipher().lock().await;
    let messages_to_send = queue
        .dispatch(cipher)
        .iter()
        .map(|m| GetMessageResponse::new(m.get_id(), m.get_content(), m.get_uuid()))
        .collect::<Vec<GetMessageResponse>>();
    HttpResponse::Accepted().json(JsonResponse::new(messages_to_send, None::<String>))
}
