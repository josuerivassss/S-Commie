use crate::{imaging::{self, text}, response::{ApiError, ApiResult}, state::AppState, validate};
use crate::extract::Query;
use axum::{extract::State, http::header, response::IntoResponse};
use image::Rgba;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct TwoWaysQuery { text1: String, text2: String, text3: String }

/// GET /image/twoways?text1=&text2=&text3= — "two ways to do X" meme with 3 captions.
pub async fn handler(State(state): State<AppState>, Query(q): Query<TwoWaysQuery>) -> ApiResult<impl IntoResponse> {
    validate::len(&q.text1, 1, 150, "text1")?;
    validate::len(&q.text2, 1, 150, "text2")?;
    validate::len(&q.text3, 1, 150, "text3")?;
    let font = state.fonts.fetch("GGSans", "Bold").map_err(|_| ApiError::internal("Font not found"))?;
    let mut image = state.images.fetch("twoways").map_err(|_| ApiError::internal("Asset not found"))?;

    let white = Rgba([255, 255, 255, 255]);
    let black = Rgba([0, 0, 0, 255]);
    let img_w = image.width() as i32;

    let centered = text::TextStyle::new(&font, 40.0).color(white).stroke(2, black).align(text::Align::Center, img_w);
    text::draw(&mut image, &state.http, &state.emojis, (0, 475), &q.text1, &centered).await;

    let plain = text::TextStyle::new(&font, 40.0).color(white).stroke(2, black);
    text::draw(&mut image, &state.http, &state.emojis, (110, 210), &text::wrap(&q.text2, 20), &plain).await;
    text::draw(&mut image, &state.http, &state.emojis, (485, 210), &text::wrap(&q.text3, 12), &plain).await;

    let bytes = imaging::prepare_png(&image)?;
    Ok(([(header::CONTENT_TYPE, "image/png")], bytes))
}