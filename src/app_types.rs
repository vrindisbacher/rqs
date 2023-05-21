use crate::queue_api::queue::Queue;
use futures::lock::{MappedMutexGuard, Mutex, MutexGuard};
use serde::Serialize;

pub struct AppState {
    pub queues: Mutex<Vec<Queue>>,
}

impl AppState {
    pub fn get_queues(&self) -> &Mutex<Vec<Queue>> {
        &self.queues
    }
    pub async fn get_queue_by_id(
        &self,
        queue_id: &String,
    ) -> Result<MappedMutexGuard<Vec<Queue>, Queue>, String> {
        let mut queues = self.get_queues().lock().await;
        match queues.iter_mut().position(|q| q.get_id() == *queue_id) {
            None => Err(format!("Could not find queue with id {}", queue_id)),
            Some(p) => Ok(MutexGuard::map(queues, |q| &mut q[p])),
        }
    }
}

#[derive(Serialize)]
pub struct JsonResponse<T, E> {
    data: T,
    error: E,
}

impl<T, E> JsonResponse<T, E> {
    pub fn new(data: T, error: E) -> JsonResponse<T, E> {
        JsonResponse { data, error }
    }
}
