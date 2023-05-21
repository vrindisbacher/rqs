use chrono::{DateTime, Duration, Utc};
use uuid::Uuid;

#[derive(Debug)]
pub struct Message {
    id: String,
    content: String,
    last_read: Option<DateTime<Utc>>,
    uuid: Uuid,
}

impl Message {
    pub fn new(id: String, content: String) -> Message {
        Message {
            id,
            content,
            last_read: None,
            uuid: Uuid::new_v4(),
        }
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

#[derive(Debug)]
pub struct Queue {
    queue: Vec<Message>, // the actual queue
    read_timeout: u32,   // the amount of time a message is hidden from consumers
    size: u32,           // should always be the same as queue.len()
    uuid: Uuid,
    id: String,
}

impl Queue {
    pub fn new(id: String, read_timeout: u32) -> Queue {
        Queue {
            queue: vec![],
            read_timeout,
            size: 0,
            uuid: Uuid::new_v4(),
            id,
        }
    }

    pub fn get_uuid(&self) -> String {
        self.uuid.to_string()
    }

    pub fn get_id(&self) -> String {
        self.id.to_owned()
    }

    pub fn add_to_queue(&mut self, id: String, content: String) -> String {
        let message = Message::new(id, content);
        let uuid = message.get_uuid();
        self.queue.push(message);
        self.incr_size();
        uuid
    }

    pub fn dispatch(&mut self) -> Option<&Message> {
        if self.size == 0 {
            return None;
        }
        for message in self.queue.iter_mut() {
            if message.is_visible(self.read_timeout) {
                message.last_read = Some(Utc::now());
                return Some(message);
            }
        }
        None
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
