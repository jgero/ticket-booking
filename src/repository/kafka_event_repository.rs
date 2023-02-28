use super::interface::{EvRepoFuture, EventRepositoryError, ProducerRepository};
use crate::model::order::Order;
use futures::StreamExt;
use log::{debug, error};
use rdkafka::{
    consumer::{Consumer, StreamConsumer},
    producer::{FutureProducer, FutureRecord},
    ClientConfig, Message,
};
use serde::{Deserialize, de::DeserializeOwned};
use std::time::Duration;
use uuid::Uuid;

const PLACED_ORDERS: &str = "placed-orders";

#[derive(Clone)]
pub struct KafkaProducerRepository {
    producer: FutureProducer,
}

impl KafkaProducerRepository {
    pub fn new(brokers: &str) -> Self {
        KafkaProducerRepository {
            producer: ClientConfig::new()
                .set("bootstrap.servers", brokers)
                .set("message.timeout.ms", "5000")
                .create()
                .expect("Producer creation error"),
        }
    }
}

impl ProducerRepository for KafkaProducerRepository {
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
