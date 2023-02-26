use super::interface::{EvRepoFuture, EventRepository, EventRepositoryError};
use crate::model::order::Order;
use log::{debug, error};
use rdkafka::{
    producer::{FutureProducer, FutureRecord},
    ClientConfig,
};
use std::time::Duration;
use uuid::Uuid;

const PLACED_ORDERS: &str = "placed-orders";

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
    fn produce_order(self, order: Order) -> EvRepoFuture<Uuid> {
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
                Ok((partition, offset)) => {
                    debug!(
                        "created message with offset {} in partition {}",
                        offset, partition
                    );
                    Ok(order.uuid)
                }
                Err((error, _owned_message)) => {
                    error!("producing order failed: {}", error);
                    Err(EventRepositoryError::from(error.to_string()))
                }
            }
        })
    }
}
