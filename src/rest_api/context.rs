use warp::Filter;

use crate::repository::message::MessageProducer;

#[derive(Clone)]
pub struct Context {
    pub message_producer: MessageProducer,
}

pub fn with_ctx(
    ctx: Context,
) -> impl Filter<Extract = (Context,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || ctx.clone())
}
