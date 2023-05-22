use crate::queue_api::queue::Queue;
use futures::lock::Mutex;
use serde::Serialize;
use std::collections::HashMap;

pub struct AppState {
    pub queues: Mutex<HashMap<String, Queue>>,
}

impl AppState {
    pub fn get_queues(&self) -> &Mutex<HashMap<String, Queue>> {
        &self.queues
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
