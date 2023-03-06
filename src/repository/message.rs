use super::errors::RepoError;
use futures::{Future, Stream, StreamExt};
use log::{debug, error};
use rdkafka::consumer::Consumer;
use rdkafka::Message;
use serde::{de::DeserializeOwned, Serialize};
use std::{pin::Pin, time::Duration};

pub type MessageConsumer = rdkafka::consumer::StreamConsumer;
pub type MessageProducer = rdkafka::producer::FutureProducer;
pub type EvRepoResult<T> = Result<T, RepoError>;
pub type EvRepoFuture<T> = Pin<Box<dyn Future<Output = EvRepoResult<T>> + Send>>;

pub enum Topic {
    PlacedOrders,
}

pub fn new_consumer(brokers: &str) -> MessageConsumer {
    rdkafka::ClientConfig::new()
        .set("group.id", "asdf")
        .set("bootstrap.servers", brokers.to_owned())
        .set("enable.partition.eof", "false")
        .set("session.timeout.ms", "6000")
        .set("enable.auto.commit", "false")
        .create()
        .expect("Consumer creation failed")
}

pub fn new_producer(brokers: &str) -> MessageProducer {
    rdkafka::ClientConfig::new()
        .set("bootstrap.servers", brokers)
        .set("message.timeout.ms", "5000")
        .create()
        .expect("Producer creation error")
}

impl Topic {
    const fn value(self) -> &'static str {
        match self {
            Self::PlacedOrders => &"placed-orders",
        }
    }

    pub fn consume<'consumer, T>(
        self,
        consumer: &'consumer rdkafka::consumer::StreamConsumer,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<T, RepoError>> + 'consumer + Send>>, RepoError>
    where
        T: DeserializeOwned,
    {
        consumer
            .subscribe(&[self.value()])
            .map_err(|e| RepoError::MessageBrokerError(e.to_string()))?;
        Ok(Box::pin(consumer.stream().map(|maybe_message| {
            maybe_message
                .map_err(|err| RepoError::MessageBrokerError(err.to_string()))
                .and_then(|message| match message.payload_view::<str>() {
                    None => Err(RepoError::DataError("message has no payload".to_string())),
                    Some(maybe_payload) => match maybe_payload {
                        Ok(payload) => Ok(payload.to_owned()),
                        Err(error) => Err(RepoError::MessageBrokerError(error.to_string())),
                    },
                })
                .and_then(|payload_str| {
                    serde_json::from_str::<T>(&payload_str)
                        .map_err(|err| RepoError::DataError(err.to_string()))
                })
        })))
    }

    pub fn produce<M>(self, producer: MessageProducer, message: M) -> EvRepoFuture<M>
    where
        M: Serialize + Send + 'static,
    {
        Box::pin(async move {
            let payload = serde_json::to_string(&message)
                .map_err(|error| RepoError::DataError(error.to_string()))?;
            let delivery_status = producer
                .send(
                    rdkafka::producer::FutureRecord::to(self.value())
                        .payload(&payload)
                        // TODO: figure out what the key is here
                        .key("todo"),
                    Duration::from_secs(5),
                )
                .await;
            match delivery_status {
                Ok((partition, offset)) => {
                    debug!(
                        "created message with offset {} in partition {}",
                        offset, partition
                    );
                    Ok(message)
                }
                Err((error, _owned_message)) => {
                    error!("producing order failed: {}", error);
                    Err(RepoError::MessageBrokerError(error.to_string()))
                }
            }
        })
    }
}
