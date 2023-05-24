use std::collections::hash_map::Entry;

use actix_web::{web, HttpResponse};
use request::{NewExchangeRequest, NewMessageRequest};

use crate::app_types::{AppState, JsonResponse};

use exchange::{Exchange, ExchangeType};
use request::ExchangeEntry;

use self::exchange::ExchangeToQueueError;

pub(crate) mod exchange;
mod request;

pub async fn new_exchange(
    data: web::Data<AppState>,
    post_data: web::Json<NewExchangeRequest>,
) -> HttpResponse {
    let mut queues = data.get_queues().lock().await;
    for queue_id in post_data.queue_ids.iter() {
        match queues.entry(queue_id.to_owned()) {
            Entry::Vacant(_) => {
                return HttpResponse::BadRequest().json(JsonResponse::new(
                    None::<String>,
                    format!("No queue with id {} was found", queue_id),
                ))
            }
            Entry::Occupied(_) => (),
        }
    }

    let mut exchanges = data.get_exchanges().lock().await;
    match exchanges.entry(post_data.id.to_owned()) {
        Entry::Vacant(_) => {
            let queue_ids = post_data.queue_ids.to_owned();
            let new_exchange =
                Exchange::new(post_data.id.to_owned(), queue_ids, &post_data.exchange_type);
            let exchange_uuid = new_exchange.uuid.to_string();
            exchanges.insert(post_data.id.to_owned(), new_exchange);
            HttpResponse::Accepted().json(JsonResponse::new(exchange_uuid, None::<String>))
        }
        Entry::Occupied(_) => HttpResponse::Conflict().json(JsonResponse::new(
            None::<String>,
            format!("An exchange with id {} already exists", &post_data.id),
        )),
    }
}

pub async fn list_exchanges(data: web::Data<AppState>) -> HttpResponse {
    let exchanges = data.get_exchanges().lock().await;
    let mut vec_of_exchanges = vec![];
    for exchange in exchanges.values() {
        vec_of_exchanges.push(ExchangeEntry {
            id: exchange.id.clone(),
            queue_ids: exchange.queue_ids.clone(),
            exchange_type: match exchange.exchange_type {
                ExchangeType::FANOUT => ExchangeType::FANOUT,
                ExchangeType::ID => ExchangeType::ID,
            },
        })
    }
    HttpResponse::Accepted().json(JsonResponse::new(vec_of_exchanges, None::<String>))
}

pub async fn add_message_to_exchange(
    data: web::Data<AppState>,
    post_data: web::Json<NewMessageRequest>,
) -> HttpResponse {
    let exchange_id = &post_data.exchange_id;

    let mut exchanges = data.get_exchanges().lock().await;
    let exchange = match exchanges.get_mut(exchange_id) {
        None => {
            return HttpResponse::BadRequest().json(JsonResponse::new(
                None::<String>,
                format!("No exchange with id {} was found", exchange_id),
            ))
        }
        Some(q) => q,
    };
    let messages_to_add = &post_data.messages;
    let mut messages_to_send = vec![];
    for message in messages_to_add.iter() {
        let id = message.message_id.to_owned();
        let content = message.content.to_owned();
        let message_added = exchange.dispatch(id, content, &data).await;
        match message_added {
            Ok(v) => messages_to_send.extend(v),
            Err(e) => match e {
                ExchangeToQueueError::NoMatchingQueueError(_) => {
                    return HttpResponse::BadRequest()
                        .json(JsonResponse::new(None::<String>, e.to_string()))
                }
                ExchangeToQueueError::UnableToAddError => {
                    return HttpResponse::InternalServerError()
                        .json(JsonResponse::new(None::<String>, e.to_string()))
                }
            },
        }
    }

    HttpResponse::Accepted().json(JsonResponse::new(messages_to_send, None::<String>))
}
