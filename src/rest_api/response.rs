use serde::Serialize;
use uuid::Uuid;
use warp::hyper::StatusCode;

use crate::repository::interface::EvRepoResult;

#[derive(Serialize)]
#[serde(untagged)]
pub enum NewOrderResponse {
    Success { uuid: Uuid },
    Failure { error: String },
}

impl warp::Reply for NewOrderResponse {
    fn into_response(self) -> warp::reply::Response {
        let mut res = warp::http::Response::new(serde_json::to_string(&self).unwrap().into());
        if let Self::Failure { error: _ } = self {
            *res.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
        }
        res
    }
}

impl From<EvRepoResult<Uuid>> for NewOrderResponse {
    fn from(value: EvRepoResult<Uuid>) -> Self {
        match value {
            Ok(uuid) => Self::Success { uuid },
            Err(err) => Self::Failure {
                error: err.to_string(),
            },
        }
    }
}
