use axum::{
    routing::{get, post},
    Router,
};
use axum_embed::ServeEmbed;
use rust_embed::Embed;

use crate::AppState;

mod index;
mod util;
mod wake;

pub(crate) use util::Base;

#[derive(Embed, Clone)]
#[folder = "assets/"]
struct Assets;

pub(crate) fn routes(state: AppState) -> axum::Router {
    let serve_assets = ServeEmbed::<Assets>::new();
    Router::new()
        .nest_service("/assets", serve_assets)
        .route("/", get(index::index))
        .route("/wake", post(wake::wake))
        .with_state(state)
}
