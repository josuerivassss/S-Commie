use crate::{imaging::{self, text}, response::{ApiError, ApiResult}, state::AppState, validate};
use crate::extract::Query;
use axum::{extract::State, http::header, response::IntoResponse};
use image::{imageops::FilterType, Rgba};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct ThisIsQuery { image: String, text: String }

/// GET /image/thisis?image=URL&text=... — "this is my playlist" card.
pub async fn handler(State(state): State<AppState>, Query(q): Query<ThisIsQuery>) -> ApiResult<impl IntoResponse> {
    validate::len(&q.text, 1, 100, "text")?;
    let font = state.fonts.fetch("FranklinGothicDemi", "Regular").map_err(|_| ApiError::internal("Font not found"))?;

    let avatar = imaging::open_image(&state.http, &q.image, "image").await?;
    let avatar = image::imageops::resize(&avatar, 345, 345, FilterType::Lanczos3);
    let avatar = imaging::ellipse_mask(avatar);

    let mut overlay = state.images.fetch("thisis").map_err(|_| ApiError::internal("Asset not found"))?;
    let colors = imaging::dominant_colors(&avatar, 2);
    imaging::draw_gradient(&mut overlay, ((0, 217), (600, 700)), &colors, true);

    let x = (overlay.width() as i64 - avatar.width() as i64) / 2;
    let y = (overlay.height() as i64 - avatar.height() as i64) / 2 + 78;
    imaging::paste(&mut overlay, &avatar, x, y);

    let overlay_w = overlay.width() as i32;
    let style = text::TextStyle::new(&font, 25.0).color(Rgba([0, 0, 0, 255])).align(text::Align::Center, overlay_w);
    text::draw(&mut overlay, &state.http, &state.emojis, (0, 115), &q.text, &style).await;

    let bytes = imaging::prepare_png(&overlay)?;
    Ok(([(header::CONTENT_TYPE, "image/png")], bytes))
}