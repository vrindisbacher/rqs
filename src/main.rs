use actix_web::{error, web, App, HttpResponse, HttpServer};
use app_types::AppState;
use general_api::ping;
use queue_api::{list_queues, new_queue};
use message_api::{add_message, get_message, delete_message};
use futures::lock::Mutex;

mod app_types;
mod general_api;
mod message_api;
mod queue_api;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let queue_data = web::Data::new(AppState {
        queues: Mutex::new(vec![]),
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
                .route("/new", web::post().to(new_queue))
            )
            .service(
                web::scope("/message")
                .route("/new", web::post().to(add_message))
                .route("/get", web::get().to(get_message))
                .route("/delete", web::post().to(delete_message))
            )

    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
