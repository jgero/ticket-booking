use std::pin::Pin;

use futures::Future;

use crate::model::order::Order;

pub struct EventRepositoryError {
    message: String,
}

impl From<String> for EventRepositoryError {
    fn from(value: String) -> Self {
        EventRepositoryError { message: value }
    }
}

pub type EventRepositoryFuture<T> = Pin<Box<dyn Future<Output = Result<T, EventRepositoryError>> + Send>>;

// TODO: using functions with `self` receiver instead of `&self` solves a lot of
// lifetime problems, but I should investigate if passing clones is bad
pub trait EventRepository: Sized + Clone {
    // returns the offset of the produced message
    fn produce_order(self, order: Order) -> EventRepositoryFuture<i64>;
}
