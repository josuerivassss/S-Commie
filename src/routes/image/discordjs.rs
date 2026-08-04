use crate::{imaging::{self, text}, response::{ApiError, ApiResult}, state::AppState, validate};
use crate::extract::Query;
use axum::{extract::State, http::header, response::IntoResponse};
use image::{imageops::FilterType, Rgba, RgbaImage};
use imageproc::{drawing::draw_filled_rect_mut, rect::Rect};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct TextQuery { text: String }

/// GET /image/discordjs?text=... — discord.js-style logo card.
pub async fn handler(State(state): State<AppState>, Query(q): Query<TextQuery>) -> ApiResult<impl IntoResponse> {
    validate::len(&q.text, 1, 55, "text")?;
    let font = state.fonts.fetch("FranklinGothicDemi", "Regular").map_err(|_| ApiError::internal("Font not found"))?;
    let overlay = state.images.fetch("js").map_err(|_| ApiError::internal("Asset not found"))?;
    let overlay = image::imageops::resize(&overlay, 165, 165, FilterType::Lanczos3);

    let upper = q.text.to_uppercase();
    let (w, h) = text::measure(&font, 85.0, &upper, 1.0);
    let (cw, ch) = ((w + 150) as u32, (h + 100) as u32);

    let mut img = RgbaImage::from_pixel(cw, ch, Rgba([9, 10, 22, 255]));
    draw_filled_rect_mut(&mut img, Rect::at(0, 0).of_size(cw, ch), Rgba([9, 10, 22, 255]));
    let mut img = imaging::rounded_mask(img, 10);

    let style = text::TextStyle::new(&font, 85.0).color(Rgba([255, 255, 255, 255]));
    text::draw(&mut img, &state.http, &state.emojis, (30, h / 2), &upper, &style).await;
    imaging::paste(&mut img, &overlay, (w - 24) as i64, (h / 4 - 12) as i64);

    let bytes = imaging::prepare_png(&img)?;
    Ok(([(header::CONTENT_TYPE, "image/png")], bytes))
}