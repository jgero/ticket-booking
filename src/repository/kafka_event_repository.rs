use std::time::Duration;

use log::{debug, error};
use rdkafka::{
    producer::{FutureProducer, FutureRecord},
    ClientConfig,
};

use crate::model::order::Order;

use super::interface::{EventRepository, EventRepositoryError, EventRepositoryFuture};

const PLACED_ORDERS: &str = "placed-orders";

pub struct KafkaEventRepositoryError {
    message: String,
}

impl From<String> for KafkaEventRepositoryError {
    fn from(value: String) -> Self {
        KafkaEventRepositoryError { message: value }
    }
}

#[derive(Clone)]
pub struct KafkaEventRepository {
    producer: FutureProducer,
}

impl KafkaEventRepository {
    pub fn new(brokers: &str) -> Self {
        KafkaEventRepository {
            producer: ClientConfig::new()
                .set("bootstrap.servers", brokers)
                .set("message.timeout.ms", "5000")
                .create()
                .expect("Producer creation error"),
        }
    }
}

impl EventRepository for KafkaEventRepository {
    fn produce_order(self, order: Order) -> EventRepositoryFuture<i64> {
        Box::pin(async move {
            let payload = serde_json::to_string(&order)
                .map_err(|error| EventRepositoryError::from(error.to_string()))?;
            let delivery_status = self
                .producer
                .send(
                    FutureRecord::to(PLACED_ORDERS)
                        .payload(&payload)
                        .key(&order.issuer),
                    Duration::from_secs(5),
                )
                .await;
            match delivery_status {
                Ok((_partition, offset)) => {
                    debug!("created message with offset {}", offset);
                    Ok(offset)
                }
                Err((error, _owned_message)) => {
                    error!("producing order {}", error);
                    Err(EventRepositoryError::from(error.to_string()))
                }
            }
        })
    }
}
