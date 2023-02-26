use warp::Filter;

use crate::repository::interface::ProducerRepository;

#[derive(Clone)]
pub struct Context<R: ProducerRepository> {
    pub ev_repo: R,
}

pub fn with_ctx<T: ProducerRepository + Clone + Sync + Send>(
    ctx: Context<T>,
) -> impl Filter<Extract = (Context<T>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || ctx.clone())
}
