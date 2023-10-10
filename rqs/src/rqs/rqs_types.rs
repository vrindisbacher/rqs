#[derive(Debug)]
pub enum RQSError {
    FailedToCreateQueue(String),
    FailedToDeleteQueue(String),
}

#[derive(Debug, Clone)]
pub enum RQSEvent {
    QueueCreated(String),
    QueueDeleted(String),
}
