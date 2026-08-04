pub mod docs;
pub mod image;
pub mod internal;
pub mod json;

use crate::state::AppState;
use axum::Router;

pub fn router() -> Router<AppState> {
    Router::new()
        .merge(json::router())
        .merge(image::router())
        .merge(internal::router())
        .merge(docs::router())
}