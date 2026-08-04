use crate::{imaging::{self, text}, response::{ApiError, ApiResult}, state::AppState, validate};
use crate::extract::Query;
use axum::{extract::State, http::header, response::IntoResponse};
use image::Rgba;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct TextQuery { text: String }

/// GET /image/sonic?text=... — sonic says meme.
pub async fn handler(State(state): State<AppState>, Query(q): Query<TextQuery>) -> ApiResult<impl IntoResponse> {
    validate::len(&q.text, 1, 150, "text")?;
    let font = state.fonts.fetch("GGSans", "Bold").map_err(|_| ApiError::internal("Font not found"))?;
    let mut image = state.images.fetch("sonic").map_err(|_| ApiError::internal("Asset not found"))?;

    let wrapped = text::wrap(&q.text, 50);
    let style = text::TextStyle::new(&font, 18.0).color(Rgba([255, 255, 255, 255]));
    text::draw(&mut image, &state.http, &state.emojis, (366, 65), &wrapped, &style).await;

    let bytes = imaging::prepare_png(&image)?;
    Ok(([(header::CONTENT_TYPE, "image/png")], bytes))
}