#[derive(Debug, PartialEq, Eq)]
pub struct RQSQueue {
    name: String,
}

impl RQSQueue {
    pub fn new(name: String) -> Self {
        Self { name }
    }

    pub fn get_name(&self) -> &String {
        &self.name
    }
}
