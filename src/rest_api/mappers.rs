use crate::model::order::{NewOrder, Order};
use log::info;
use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    net::SocketAddr,
};

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
        }
    }
}
