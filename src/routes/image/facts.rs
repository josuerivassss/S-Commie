use crate::{imaging::{self, text}, response::{ApiError, ApiResult}, state::AppState, validate};
use crate::extract::Query;
use axum::{extract::State, http::header, response::IntoResponse};
use image::{Rgba, RgbaImage};
use imageproc::geometric_transformations::{rotate_about_center, Interpolation};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct TextQuery { text: String }

/// GET /image/facts?text=... — "facts" meme, text rotated -15 degrees.
pub async fn handler(State(state): State<AppState>, Query(q): Query<TextQuery>) -> ApiResult<impl IntoResponse> {
    validate::len(&q.text, 1, 100, "text")?;
    let font = state.fonts.fetch("GGSans", "Medium").map_err(|_| ApiError::internal("Font not found"))?;
    let mut image = state.images.fetch("facts").map_err(|_| ApiError::internal("Asset not found"))?;

    let wrapped = text::wrap(&q.text, 22);
    let mut layer = RgbaImage::from_pixel(image.width(), image.height(), Rgba([0, 0, 0, 0]));
    let style = text::TextStyle::new(&font, 22.0).color(Rgba([0, 0, 0, 255]));
    text::draw(&mut layer, &state.http, &state.emojis, (75, 400), &wrapped, &style).await;
    let layer = rotate_about_center(&layer, (-15f32).to_radians(), Interpolation::Bilinear, Rgba([0, 0, 0, 0]));

    imaging::paste(&mut image, &layer, 0, 0);
    let bytes = imaging::prepare_png(&image)?;
    Ok(([(header::CONTENT_TYPE, "image/png")], bytes))
}