use crate::{imaging::{self, text}, response::{ApiError, ApiResult}, state::AppState, validate};
use crate::extract::Query;
use axum::{extract::State, http::header, response::IntoResponse};
use image::{Rgba, RgbaImage};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct TextQuery { text: String }

/// GET /image/supreme?text=... — supreme-style logo card.
pub async fn handler(State(state): State<AppState>, Query(q): Query<TextQuery>) -> ApiResult<impl IntoResponse> {
    validate::len(&q.text, 1, 55, "text")?;
    let font = state.fonts.fetch("HelveticaNow", "Regular").map_err(|_| ApiError::internal("Font not found"))?;

    let (w, h) = text::measure(&font, 60.0, &q.text, 1.0);
    let (cw, ch) = ((w + 30) as u32, (h + 20) as u32);

    let img = RgbaImage::from_pixel(cw, ch, Rgba([255, 0, 0, 255]));
    let mut img = imaging::rounded_mask(img, 10);

    let style = text::TextStyle::new(&font, 60.0).color(Rgba([255, 255, 255, 255]));
    text::draw(&mut img, &state.http, &state.emojis, (15, 2), &q.text, &style).await;

    let bytes = imaging::prepare_png(&img)?;
    Ok(([(header::CONTENT_TYPE, "image/png")], bytes))
}