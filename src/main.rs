use crate::model::order::{NewOrder, Order};
use crate::repository::interface::EventRepository;
use crate::repository::kafka_event_repository::KafkaEventRepository;
use crate::rest_api::context::with_ctx;
use crate::rest_api::context::Context;
use crate::rest_api::mappers::with_issuer;
use crate::rest_api::response::NewOrderResponse;
use log::info;
use logging_utils::setup_logger;
use warp::Filter;

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

    // NOTES TO SELF:
    // `and` and `and_then` reject on error, which means it could be `recover`able
    // or be handled by `or`.
    // `then` on the other hand does not care about all that, it acts like an async
    // verison of `map`
    let new_order = warp::path("order")
        .and(warp::post())
        .and(warp::body::content_length_limit(4096))
        .and(warp::body::json::<NewOrder>())
        .and(warp::addr::remote())
        .and_then(with_issuer)
        .and(with_ctx(ctx.clone()))
        .then(
            |order: Order, ctx: Context<KafkaEventRepository>| async move {
                ctx.ev_repo.clone().produce_order(order.clone()).await
            },
        )
        .map(NewOrderResponse::from);

    let hello = warp::path("hello")
        .and(warp::path::param())
        .and(warp::header("user-agent"))
        .map(|param: String, agent: String| format!("Hello {}, whose agent is {}", param, agent))
        .with(warp::log("request-log"));
    warp::serve(new_order.or(hello))
        .run(([127, 0, 0, 1], 8000))
        .await;
}
