use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    net::SocketAddr,
};

use log::info;

use crate::model::order::{NewOrder, Order};
use logging_utils::setup_logger;

use warp::{
    http::Response,
    reject::{self, Reject},
    Filter,
};

mod logging_utils;
mod model;

#[derive(Debug)]
struct NoSocketAddr;
impl Reject for NoSocketAddr {}

#[tokio::main]
async fn main() {
    setup_logger(true, None);
    info!("starting application");

    let new_order = warp::path("order")
        .and(warp::post())
        .and(warp::body::content_length_limit(4096))
        .and(warp::body::json::<NewOrder>())
        .and(warp::addr::remote())
        .and_then(|order: NewOrder, addr: Option<SocketAddr>| async move {
            let addr_hash = addr.map(|v| {
                let mut s = DefaultHasher::new();
                v.hash(&mut s);
                s.finish().to_string()
            });
            match addr_hash {
                None => Err(reject::custom(NoSocketAddr)),
                Some(issuer) => Ok(Order::new(order, issuer)),
            }
        })
        .map(|order: Order| Response::builder().body(order.to_string()));

    let hello = warp::path("hello")
        .and(warp::path::param())
        .and(warp::header("user-agent"))
        .map(|param: String, agent: String| format!("Hello {}, whose agent is {}", param, agent))
        .with(warp::log("request-log"));
    warp::serve(new_order.or(hello))
        .run(([127, 0, 0, 1], 8000))
        .await;
    // TODO: how to do querying. possible to query kafka event log?
}
