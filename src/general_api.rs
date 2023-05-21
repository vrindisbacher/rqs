use actix_web::HttpResponse;

pub async fn ping() -> HttpResponse {
    HttpResponse::Accepted().body("pong")
}
