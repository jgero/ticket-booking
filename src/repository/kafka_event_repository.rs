use super::interface::{EvRepoFuture, EventRepositoryError, ProducerRepository};
use crate::model::order::Order;
use futures::StreamExt;
use log::{debug, error};
use rdkafka::{
    consumer::{Consumer, StreamConsumer},
    producer::{FutureProducer, FutureRecord},
    ClientConfig, Message,
};
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

pub struct KafkaConsumer {
    consumer: StreamConsumer,
}

impl KafkaConsumer {
    pub fn new(brokers: String) -> Self {
        KafkaConsumer {
            consumer: ClientConfig::new()
                .set("group.id", "asdf")
                .set("bootstrap.servers", brokers.to_owned())
                .set("enable.partition.eof", "false")
                .set("session.timeout.ms", "6000")
                .set("enable.auto.commit", "false")
                .create()
                .expect("Consumer creation failed"),
        }
    }
    pub async fn consume_orders(self, callback: Box<dyn Fn(Order) + Send + Sync>) {
        self.consumer
            .subscribe(&[PLACED_ORDERS])
            .expect("consumer subscription error");
        self.consumer
            .stream()
            .map(|maybe_message| {
                maybe_message
                    .map_err(|err| EventRepositoryError::from(err.to_string()))
                    .and_then(|message| match message.payload_view::<str>() {
                        None => Err(EventRepositoryError::from(
                            "message has no payload".to_string(),
                        )),
                        Some(maybe_payload) => match maybe_payload {
                            Ok(payload) => Ok(payload.to_owned()),
                            Err(error) => Err(EventRepositoryError::from(error.to_string())),
                        },
                    })
                    .and_then(|payload_str| {
                        serde_json::from_str::<Order>(&payload_str)
                            .map_err(|err| EventRepositoryError::from(err.to_string()))
                    })
            })
            .for_each(|maybe_order| async {
                match maybe_order {
                    Ok(order) => callback(order),
                    Err(error) => error!("coud not consume order: {}", error.to_string()),
                }
            })
            .await;
    }
}
