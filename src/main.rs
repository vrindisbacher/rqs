use actix_web::{error, web, App, HttpResponse, HttpServer};
use aes_gcm::{
    aead::{KeyInit, OsRng},
    Aes256Gcm,
};
use app_types::AppState;
use exchange_api::{add_message_to_exchange, list_exchanges, new_exchange};
use futures::lock::Mutex;
use general_api::ping;
use message_api::{add_message_to_queue, delete_message, get_message};
use queue_api::{list_queues, new_queue};
use std::collections::HashMap;

mod app_types;
mod exchange_api;
mod general_api;
mod message_api;
mod queue_api;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // generate unique key on start up
    let key = Aes256Gcm::generate_key(&mut OsRng);
    let cipher = Aes256Gcm::new(&key);

    let queue_data = web::Data::new(AppState {
        queues: Mutex::new(HashMap::new()),
        exchanges: Mutex::new(HashMap::new()),
        cipher: Mutex::new(cipher),
    });

    HttpServer::new(move || {
        let json_config = web::JsonConfig::default()
            .limit(4096)
            .error_handler(|err, _req| {
                // create custom error response
                error::InternalError::from_response(
                    err,
                    HttpResponse::BadRequest().body("JSON was malformed"),
                )
                .into()
            });

        App::new()
            .app_data(json_config)
            .app_data(queue_data.clone())
            .route("/", web::get().to(ping))
            .service(
                web::scope("/queue")
                    .route("/list", web::get().to(list_queues))
                    .route("/new", web::post().to(new_queue)),
            )
            .service(
                web::scope("/message")
                    .route("/new", web::post().to(add_message_to_queue))
                    .route("/get", web::get().to(get_message))
                    .route("/delete", web::post().to(delete_message)),
            )
            .service(
                web::scope("/exchange")
                    .route("/list", web::get().to(list_exchanges))
                    .route("/new", web::post().to(new_exchange))
                    .route("/add", web::post().to(add_message_to_exchange)),
            )
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
