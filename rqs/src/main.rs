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

#[cfg(test)]
mod queue_client_server_test {
    use std::time::Duration;

    use crate::rqs::{EVENT_LOG, LOG_ROOT, RQS};
    use crate::{
        message::Message,
        queue::{
            queue::{queue_service_client::QueueServiceClient, NewQueueRequest},
            Queue,
        },
        GLOBAL_DATA,
    };
    use serial_test::serial;
    use tonic::transport::Server;

    use super::queue::queue::DeleteQueueRequest;

    async fn start() {
        delete_event_log();
        let mut rqs = GLOBAL_DATA.lock().await;
        rqs.clear();
        rqs.revive_from_log().await;
        spawn_server().await;
    }

    async fn spawn_server() {
        // totally hacky way of starting up the server
        tokio::spawn(async {
            let server_addr = "127.0.0.1:8080".parse().unwrap();
            Server::builder()
                .add_service(tonic_web::enable(Message::new_message_server()))
                .add_service(tonic_web::enable(Queue::new_queue_server()))
                .serve(server_addr)
                .await
                .unwrap()
        });
        tokio::time::sleep(Duration::from_secs(3)).await;
    }

    fn delete_event_log() {
        let _ = std::fs::remove_file(format!("{LOG_ROOT}{EVENT_LOG}"));
    }

    #[tokio::test]
    #[serial]
    async fn test_create_queue_request() {
        start().await;

        let client_addr = "http://127.0.0.1:8080";
        let mut client = QueueServiceClient::connect(client_addr)
            .await
            .expect("Could not create client");
        let request = NewQueueRequest {
            queue_id: "queue_1".to_string(),
            visibility_timeout: 5,
        };
        client
            .new_queue(request)
            .await
            .expect("Failed to create queue request");

        let rqs = GLOBAL_DATA.lock().await;
        let queues = rqs
            .get_queues()
            .iter()
            .map(|x| x.get_name())
            .collect::<Vec<&String>>();
        assert_eq!(queues, vec!["queue_1"]);
    }

    #[tokio::test]
    #[serial]
    async fn test_create_and_delete_queue_request() {
        start().await;

        let client_addr = "http://127.0.0.1:8080";
        let mut client = QueueServiceClient::connect(client_addr)
            .await
            .expect("Could not create client");
        let request = NewQueueRequest {
            queue_id: "queue_1".to_string(),
            visibility_timeout: 5,
        };
        client
            .new_queue(request)
            .await
            .expect("Failed to create queue request");

        let request = DeleteQueueRequest {
            queue_id: "queue_1".to_string(),
        };
        client
            .delete_queue(request)
            .await
            .expect("Failed to delete queue");

        let rqs = GLOBAL_DATA.lock().await;
        let queues = rqs
            .get_queues()
            .iter()
            .map(|x| x.get_name())
            .collect::<Vec<&String>>();
        assert_eq!(queues, vec![] as Vec<&String>);
    }

    #[tokio::test]
    #[serial]
    async fn test_multiple_concurrent_queue_requests() {
        start().await;
        let client_addr = "http://127.0.0.1:8080";
        let mut client = QueueServiceClient::connect(client_addr)
            .await
            .expect("Could not create client");
        let request1 = NewQueueRequest {
            queue_id: "queue_1".to_string(),
            visibility_timeout: 5,
        };
        let request2 = NewQueueRequest {
            queue_id: "queue_2".to_string(),
            visibility_timeout: 5,
        };
        futures::future::join_all([
            client.clone().new_queue(request1),
            client.new_queue(request2),
        ])
        .await;

        // should have two queues
        let rqs = GLOBAL_DATA.lock().await;
        let queues = rqs
            .get_queues()
            .iter()
            .map(|x| x.get_name())
            .collect::<Vec<&String>>();
        assert_eq!(queues, vec!["queue_1", "queue_2"]);

        // should also be able to revive the queues
        let mut rqs = RQS::new();
        rqs.revive_from_log().await;
        let queues = rqs
            .get_queues()
            .iter()
            .map(|x| x.get_name())
            .collect::<Vec<&String>>();
        assert_eq!(queues, vec!["queue_1", "queue_2"]);
    }
}
