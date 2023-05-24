use std::error::Error;

use aes_gcm::aead::Aead;
use aes_gcm::aes::Aes256;
use aes_gcm::AesGcm;
use chrono::{DateTime, Duration, Utc};
use futures::lock::MutexGuard;
use uuid::Uuid;

use aes_gcm::aes::cipher::typenum::bit::{B0, B1};
use aes_gcm::aes::cipher::typenum::{UInt, UTerm};
use aes_gcm::{
    aead::{generic_array::GenericArray, AeadCore, OsRng},
    Aes256Gcm,
};

#[derive(Debug)]
pub struct EncryptionError;

#[derive(Debug)]
pub struct Message {
    id: String,
    content: Vec<u8>,
    last_read: Option<DateTime<Utc>>,
    uuid: Uuid,
    nonce: GenericArray<u8, UInt<UInt<UInt<UInt<UTerm, B1>, B1>, B0>, B0>>,
}

impl Message {
    pub fn new(
        id: String,
        content: Vec<u8>,
        nonce: GenericArray<u8, UInt<UInt<UInt<UInt<UTerm, B1>, B1>, B0>, B0>>,
    ) -> Self {
        Message {
            id,
            content,
            last_read: None,
            uuid: Uuid::new_v4(),
            nonce,
        }
    }

    pub fn get_uuid(&self) -> String {
        self.uuid.to_string()
    }

    pub fn uuid_matches(&self, uuid: &String) -> bool {
        self.uuid.to_string() == *uuid
    }

    pub fn is_visible(&self, read_timeout: u32) -> bool {
        match self.last_read {
            None => true,
            Some(dt) => {
                let now = Utc::now();
                now - dt > Duration::seconds(read_timeout as i64)
            }
        }
    }
}

pub struct DecryptedMessage {
    id: String,
    content: String,
    uuid: Uuid,
}

impl DecryptedMessage {
    pub fn new(id: String, content: String, uuid: Uuid) -> Self {
        DecryptedMessage { id, content, uuid }
    }
    pub fn get_uuid(&self) -> String {
        self.uuid.to_string()
    }

    pub fn get_id(&self) -> String {
        (*self.id).to_owned()
    }

    pub fn get_content(&self) -> String {
        (*self.content).to_owned()
    }
}

#[derive(Debug)]
pub struct Queue {
    queue: Vec<Message>, // the actual queue
    read_timeout: u32,   // the amount of time a message is hidden from consumers
    size: u32,           // should always be the same as queue.len()
    uuid: Uuid,          // unique uuid
    id: String,          // user id for queue - also unique
    max_batch: u32,      // the max number of messages to insert and return at once
}

impl Queue {
    pub fn new(id: String, read_timeout: u32, max_batch: u32) -> Self {
        Queue {
            queue: vec![],
            read_timeout,
            size: 0,
            uuid: Uuid::new_v4(),
            id,
            max_batch,
        }
    }

    pub fn get_uuid(&self) -> String {
        self.uuid.to_string()
    }

    pub fn get_id(&self) -> String {
        self.id.to_owned()
    }

    pub fn add_to_queue(
        &mut self,
        cipher: &MutexGuard<AesGcm<Aes256, UInt<UInt<UInt<UInt<UTerm, B1>, B1>, B0>, B0>>>,
        id: String,
        content: String,
    ) -> Result<String, EncryptionError> {
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng); // 96-bits; unique per message
        let ciphered_content = match cipher.encrypt(&nonce, content.as_ref()) {
            Ok(s) => s,
            Err(_) => return Err(EncryptionError),
        };
        let message = Message::new(id, ciphered_content, nonce);
        let uuid = message.get_uuid();
        self.queue.push(message);
        self.incr_size();
        Ok(uuid)
    }

    pub fn dispatch(
        &mut self,
        cipher: MutexGuard<AesGcm<Aes256, UInt<UInt<UInt<UInt<UTerm, B1>, B1>, B0>, B0>>>,
    ) -> Result<Vec<DecryptedMessage>, EncryptionError> {
        if self.size == 0 {
            return Ok(vec![]);
        }
        let mut messages_to_dispatch = vec![];
        for message in self.queue.iter_mut() {
            if message.is_visible(self.read_timeout) {
                // uncipher the message
                let unciphered_content =
                    match cipher.decrypt(&message.nonce, message.content.as_ref()) {
                        Ok(s) => s,
                        Err(_) => return Err(EncryptionError),
                    };
                let content = match String::from_utf8(unciphered_content) {
                    Ok(s) => s,
                    Err(_) => return Err(EncryptionError),
                };

                let decrypted_message =
                    DecryptedMessage::new(message.id.clone(), content, message.uuid);
                message.last_read = Some(Utc::now());
                messages_to_dispatch.push(decrypted_message);
            }
            if messages_to_dispatch.len() == self.max_batch as usize {
                break;
            }
        }
        Ok(messages_to_dispatch)
    }

    pub fn rem_from_queue(&mut self, uuid: &String) -> Option<Message> {
        if self.size == 0 {
            return None;
        }
        for (idx, message) in self.queue.iter().enumerate() {
            if message.uuid_matches(uuid) && !message.is_visible(self.read_timeout) {
                let message_to_return = self.queue.remove(idx);
                self.decr_size();
                return Some(message_to_return);
            }
        }
        None
    }

    fn incr_size(&mut self) {
        self.size += 1;
    }

    fn decr_size(&mut self) {
        self.size -= 1;
    }
}
