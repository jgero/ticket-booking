use std::fmt::Display;

pub enum RepoError {
    MessageBrokerError(String),
    DataError(String),
    DatabaseError(String),
}

impl Display for RepoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MessageBrokerError(msg) => write!(f, "MessageBrokerError: {}", msg),
            Self::DataError(msg) => write!(f, "DataError: {}", msg),
            Self::DatabaseError(msg) => write!(f, "DatabaseError: {}", msg),
        }
    }
}
