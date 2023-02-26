use super::interface::{EvRepoFuture, ProducerRepository, EventRepositoryError, ConsumerRepository};
use crate::{model::order::Order, rest_api::context::Context};
use futures::TryStreamExt;
use log::{debug, error, info};
use rdkafka::{
    consumer::{StreamConsumer, Consumer, self},
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

pub async fn handle_orders(brokers: String) {
    let consumer: StreamConsumer = ClientConfig::new()
        .set("group.id", "asdf")
        .set("bootstrap.servers", brokers.to_owned())
        .set("enable.partition.eof", "false")
        .set("session.timeout.ms", "6000")
        .set("enable.auto.commit", "false")
        .create()
        .expect("Consumer creation failed");
    consumer.subscribe(&[ PLACED_ORDERS ]).expect("consumer creation error");
    consumer.stream().try_for_each(|borrowed_message| {
        async move {
            match borrowed_message.payload_view::<str>() {
                Some(Ok(payload)) => info!("received placed orders message: {}", payload),
                Some(Err(err)) => error!("message is no string: {}", err),
                None => error!("message has no payload")
            }
            Ok(())
        }
    }).await.expect("could not subscribe to topic");
}
