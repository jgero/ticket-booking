use crate::{
    model::order::Order,
    repository::{
        message::{new_consumer, Topic},
        storage::realize_order,
    },
};
use futures::StreamExt;
use log::{error, info};
use sqlx::{Pool, Postgres};
use std::sync::Arc;

pub async fn handle_placed_orders(brokers: &str, pool: Pool<Postgres>) {
    let consumer = Arc::new(new_consumer(brokers));
    if let Ok(stream) = Topic::PlacedOrders.consume::<Order>(&consumer.clone()) {
        stream
            .for_each(|maybe_order| async {
                if let Ok(order) = maybe_order {
                    handle_message(order, pool.clone()).await;
                }
            })
            .await;
    }
}

async fn handle_message(order: Order, pool: Pool<Postgres>) {
    match realize_order(order.clone(), pool).await {
        Ok(was_successful) => {
            if !was_successful {
                info!("could not realize order {}", order.uuid.to_string());
            }
        }
        Err(error) => {
            error!("could not realize order: {}", error.to_string());
        }
    }
}
