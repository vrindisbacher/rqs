use message::Message;
use queue::Queue;
use rqs::RQS;
use tokio::sync::Mutex;
use tonic::transport::Server;

use once_cell::sync::Lazy;

pub mod message;
pub mod queue;
mod rqs;

static GLOBAL_DATA: Lazy<Mutex<RQS>> = Lazy::new(|| Mutex::new(RQS::new()));

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "127.0.0.1:8080".parse().unwrap();
    let mut rqs_state = GLOBAL_DATA.lock().await;
    rqs_state.revive_from_log().await;
    Server::builder()
        .add_service(tonic_web::enable(Message::new_message_server()))
        .add_service(tonic_web::enable(Queue::new_queue_server()))
        .serve(addr)
        .await?;

    Ok(())
}

