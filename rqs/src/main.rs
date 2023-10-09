use message::Message;
use queue::Queue;
use rqs::RQS;
use tonic::transport::Server;

mod message; 
mod queue; 
mod rqs;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "127.0.0.1:8080".parse().unwrap();
    let rqs = RQS::new();
    Server::builder()
        .add_service(tonic_web::enable(Message::new_message_server()))
        .add_service(tonic_web::enable(Queue::new_queue_server()))
        .serve(addr)
        .await?;

    Ok(())
}
