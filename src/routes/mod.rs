pub mod docs;
pub mod image;
pub mod internal;
pub mod json;

use crate::state::AppState;
use axum::Router;

pub fn router(state: AppState) -> Router<AppState> {
    Router::new()
        .merge(json::router(state))
        .merge(image::router())
        .merge(internal::router())
        .merge(docs::router())
}