use crate::model::order::Order;
use futures::Future;
use std::{fmt::Display, pin::Pin};
use uuid::Uuid;

#[derive(Debug)]
pub struct EventRepositoryError {
    message: String,
}

impl Display for EventRepositoryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "EventRepositoryError: {}", self.message)
    }
}

impl From<String> for EventRepositoryError {
    fn from(value: String) -> Self {
        EventRepositoryError { message: value }
    }
}

pub type EvRepoResult<T> = Result<T, EventRepositoryError>;
pub type EvRepoFuture<T> = Pin<Box<dyn Future<Output = EvRepoResult<T>> + Send>>;

// TODO: using functions with `self` receiver instead of `&self` solves a lot of
// lifetime problems, but I should investigate if passing clones is bad
pub trait EventRepository: Sized + Clone {
    // returns the offset of the produced message
    fn produce_order(self, order: Order) -> EvRepoFuture<Uuid>;
}