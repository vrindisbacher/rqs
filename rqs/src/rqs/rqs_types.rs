use std::fmt::Display;

#[derive(Debug)]
pub enum RQSError {
    FailedToCreateQueue(String),
    FailedToDeleteQueue(String),
}

impl Display for RQSError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RQSError::FailedToCreateQueue(s) => write!(f, "{}", s),
            RQSError::FailedToDeleteQueue(s) => write!(f, "{}", s),
        }
    }
}

#[derive(Debug, Clone)]
pub enum RQSEvent {
    QueueCreated(String),
    QueueDeleted(String),
}
