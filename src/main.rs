use log::info;

use crate::model::order::{NewOrder, Order};
use crate::repository::kafka_event_repository::KafkaEventRepository;
use crate::rest_api::context::with_ctx;
use crate::rest_api::context::Context;
use crate::rest_api::mappers::produce_order;
use crate::rest_api::mappers::with_issuer;
use logging_utils::setup_logger;

use warp::{http::Response, Filter};

mod logging_utils;
mod model;
mod repository;
mod rest_api;

#[tokio::main]
async fn main() {
    setup_logger(true, None);
    info!("starting application");

    let ctx: &Context<KafkaEventRepository> = &Context {
        ev_repo: KafkaEventRepository::new("localhost:9092"),
    };

    let new_order = warp::path("order")
        .and(warp::post())
        .and(warp::body::content_length_limit(4096))
        .and(warp::body::json::<NewOrder>())
        .and(warp::addr::remote())
        .and_then(with_issuer)
        .and(with_ctx(ctx.clone()))
        .and_then(produce_order)
        .map(|order: Order| Response::builder().body(serde_json::to_string(&order).unwrap()));

    let hello = warp::path("hello")
        .and(warp::path::param())
        .and(warp::header("user-agent"))
        .map(|param: String, agent: String| format!("Hello {}, whose agent is {}", param, agent))
        .with(warp::log("request-log"));
    warp::serve(new_order.or(hello))
        .run(([127, 0, 0, 1], 8000))
        .await;
}
