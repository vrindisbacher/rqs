use crate::exchange_api::exchange::Exchange;
use crate::queue_api::queue::Queue;
use aes_gcm::aes::cipher::typenum::bit::{B0, B1};
use aes_gcm::aes::cipher::typenum::{UInt, UTerm};
use aes_gcm::aes::Aes256;
use aes_gcm::AesGcm;
use futures::lock::Mutex;
use serde::Serialize;
use std::collections::HashMap;

pub struct AppState {
    pub queues: Mutex<HashMap<String, Queue>>,
    pub exchanges: Mutex<HashMap<String, Exchange>>,
    pub cipher: Mutex<AesGcm<Aes256, UInt<UInt<UInt<UInt<UTerm, B1>, B1>, B0>, B0>>>,
}

impl AppState {
    pub fn get_queues(&self) -> &Mutex<HashMap<String, Queue>> {
        &self.queues
    }
    pub fn get_exchanges(&self) -> &Mutex<HashMap<String, Exchange>> {
        &self.exchanges
    }
    pub fn get_cipher(
        &self,
    ) -> &Mutex<AesGcm<Aes256, UInt<UInt<UInt<UInt<UTerm, B1>, B1>, B0>, B0>>> {
        &self.cipher
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
