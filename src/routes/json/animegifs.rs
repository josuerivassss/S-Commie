use crate::response::{ApiError, ApiOk, ApiResult};
use crate::extract::Query;
use crate::state::AppState;
use axum::extract::State;
use rand::seq::SliceRandom;
use serde::Deserialize;

const STYLES: [&str; 15] = [
    "angry", "baka", "bite", "blush", "cry", "dance", "deredere", "happy",
    "hug", "kiss", "path", "punch", "slap", "sleep", "smug",
];

#[derive(Deserialize)]
pub struct GifQuery {
    style: String,
}

/// GET /json/animegifs?style=... — returns a random gif URL for the requested reaction.
pub async fn handler(State(state): State<AppState>, Query(q): Query<GifQuery>) -> ApiResult {
    if !STYLES.contains(&q.style.as_str()) {
        return Err(ApiError::validation(format!("style must be one of: {}", STYLES.join(", ")), "style"));
    }

    let options = state
        .gifs
        .get(&q.style)
        .filter(|v| !v.is_empty())
        .ok_or_else(ApiError::not_found)?;

    let gif = options.choose(&mut rand::thread_rng()).unwrap();
    Ok(ApiOk::new(gif))
}
