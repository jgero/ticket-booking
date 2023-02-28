use super::interface::EventRepositoryError;
use futures::StreamExt;
use log::error;
use rdkafka::{consumer::Consumer, Message};

type ConsumerCallback<T> = Box<dyn Fn(T) + Send + Sync>;

pub enum Topic {
    PlacedOrders,
}

impl Topic {
    const fn value(self) -> &'static str {
        match self {
            Self::PlacedOrders => &"placed-orders",
        }
    }

    pub async fn consume<T>(self, brokers: String, callback: ConsumerCallback<T>)
    where
        T: serde::de::DeserializeOwned,
    {
        let consumer: rdkafka::consumer::StreamConsumer = rdkafka::ClientConfig::new()
            .set("group.id", "asdf")
            .set("bootstrap.servers", brokers.to_owned())
            .set("enable.partition.eof", "false")
            .set("session.timeout.ms", "6000")
            .set("enable.auto.commit", "false")
            .create()
            .expect("Consumer creation failed");
        consumer
            .subscribe(&[self.value()])
            .expect("consumer subscription error");
        consumer
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
                        serde_json::from_str::<T>(&payload_str)
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
