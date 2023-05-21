use crate::app_types::{AppState, JsonResponse};
use actix_web::{web, HttpResponse};
use request::{DeleteMessageRequest, GetMessageRequest, GetMessageResponse, NewMessageRequest};

mod request;

pub async fn add_message(
    data: web::Data<AppState>,
    post_data: web::Json<NewMessageRequest>,
) -> HttpResponse {
    let queue_id = &post_data.queue_id;

    let mut queue = match data.get_queue_by_id(queue_id).await {
        Ok(q) => q,
        Err(e) => return HttpResponse::BadRequest().json(JsonResponse::new(None::<String>, e)),
    };

    let message_id = post_data.message_id.to_owned();
    let message_content = post_data.content.to_owned();
    let new_message_uuid = queue.add_to_queue(message_id, message_content);

    HttpResponse::Accepted().json(JsonResponse::new(new_message_uuid, None::<String>))
}

pub async fn delete_message(
    data: web::Data<AppState>,
    post_data: web::Json<DeleteMessageRequest>,
) -> HttpResponse {
    let queue_id = &post_data.queue_id;

    let mut queue = match data.get_queue_by_id(queue_id).await {
        Ok(q) => q,
        Err(e) => return HttpResponse::BadRequest().json(JsonResponse::new(None::<String>, e)),
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

    let mut queue = match data.get_queue_by_id(queue_id).await {
        Ok(q) => q,
        Err(e) => return HttpResponse::BadRequest().json(JsonResponse::new(None::<String>, e)),
    };

    match queue.dispatch() {
        None => HttpResponse::Accepted().json(JsonResponse::new(None::<String>, None::<String>)),
        Some(m) => HttpResponse::Accepted().json(JsonResponse::new(
            GetMessageResponse::new(m.get_id(), m.get_content(), m.get_uuid()),
            None::<String>,
        )),
    }
}
