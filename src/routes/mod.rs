use axum::{
    routing::{get, post},
    Router,
};

use crate::AppState;

mod index;
mod wake;

pub(crate) fn routes(state: AppState) -> axum::Router {
    Router::new()
        .route("/", get(index::index))
        .route("/wake", post(wake::wake))
        .with_state(state)
}
