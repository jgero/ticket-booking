use warp::Filter;

use crate::repository::{interface::EventRepository, kafka_event_repository::KafkaEventRepository};

#[derive(Clone)]
pub struct Context<R: EventRepository> {
    pub ev_repo: R,
}

pub fn with_ctx<T: EventRepository + Clone + Sync + Send>(
    ctx: Context<T>,
) -> impl Filter<Extract = (Context<T>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || ctx.clone())
}
