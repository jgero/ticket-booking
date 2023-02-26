use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    net::SocketAddr,
};

use log::info;

use crate::{model::order::{NewOrder, Order}, repository::{kafka_event_repository::KafkaEventRepository, interface::EventRepository}};

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
        v.ip().hash(&mut s);
        s.finish().to_string()
    });
    match addr_hash {
        None => Err(warp::reject::custom(NoSocketAddr)),
        Some(issuer) => {
            info!("issuer is {}", issuer);
            Ok(Order::new(order, issuer))
        },
    }
}

#[derive(Debug)]
struct CouldNotPersist;
impl warp::reject::Reject for CouldNotPersist {}

pub async fn produce_order<T : EventRepository>(order: Order, ctx: Context<T>
) -> Result<Order, warp::Rejection> {
    match ctx.ev_repo.clone().produce_order(order.clone()).await {
        Err(error) => Err(warp::reject::custom(CouldNotPersist)),
        Ok(_) => Ok(order),
    }
}
