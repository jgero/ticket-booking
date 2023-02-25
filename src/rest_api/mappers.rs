use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    net::SocketAddr,
};

use crate::{model::order::{NewOrder, Order}, repository::kafka_event_repository::KafkaEventRepository};

use super::context::Context;

#[derive(Debug)]
pub struct NoSocketAddr;
impl warp::reject::Reject for NoSocketAddr {}

pub async fn with_issuer(
    order: NewOrder,
    addr: Option<SocketAddr>,
) -> Result<Order, warp::Rejection> {
    let addr_hash = addr.map(|v| {
        let mut s = DefaultHasher::new();
        v.hash(&mut s);
        s.finish().to_string()
    });
    match addr_hash {
        None => Err(warp::reject::custom(NoSocketAddr)),
        Some(issuer) => Ok(Order::new(order, issuer)),
    }
}

#[derive(Debug)]
struct CouldNotPersist;
impl warp::reject::Reject for CouldNotPersist {}

pub async fn produce_order(order: Order, ctx: Context<KafkaEventRepository>
) -> Result<Order, warp::Rejection> {
    match ctx.ev_repo.produce_order(&order).await {
        Err(error) => Err(warp::reject::custom(CouldNotPersist)),
        Ok(_) => Ok(order),
    }
}
