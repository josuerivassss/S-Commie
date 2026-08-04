mod animegifs;
mod binary;
mod calendar;
mod eightball;
mod ocr;

use crate::state::AppState;
use axum::{routing::get, Router};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/json/binary", get(binary::handler))
        .route("/json/8ball", get(eightball::handler))
        .route("/json/animegifs", get(animegifs::handler))
        .route("/json/calendar", get(calendar::handler))
        .route("/json/ocr", get(ocr::handler))
}