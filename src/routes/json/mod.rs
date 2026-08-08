mod animegifs;
mod binary;
mod calendar;
mod eightball;
mod imagesearch;
mod ocr;
mod translate;
mod weather;

use crate::{apikeys::api_key_auth, state::AppState};
use axum::{middleware, routing::get, Router};

pub fn router(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/json/binary", get(binary::handler))
        .route("/json/8ball", get(eightball::handler))
        .route("/json/animegifs", get(animegifs::handler))
        .route("/json/calendar", get(calendar::handler))
        .route("/json/ocr", get(ocr::handler))
        .route("/json/translate", get(translate::handler))
        .route("/json/weather", get(weather::handler))
        .route("/json/imagesearch", get(imagesearch::handler))
        .layer(middleware::from_fn_with_state(state, api_key_auth))
}