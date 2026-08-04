use crate::{imaging::{self, text}, response::{ApiError, ApiResult}, state::AppState, validate};
use crate::extract::Query;
use axum::{extract::State, http::header, response::IntoResponse};
use image::Rgba;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct TextQuery { text: String }

/// GET /image/badnews?text=... — "guys i have bad news" meme.
pub async fn handler(State(state): State<AppState>, Query(q): Query<TextQuery>) -> ApiResult<impl IntoResponse> {
    validate::len(&q.text, 1, 130, "text")?;
    let font = state.fonts.fetch("FranklinGothicDemi", "Regular").map_err(|_| ApiError::internal("Font not found"))?;
    let mut img = state.images.fetch("gru").map_err(|_| ApiError::internal("Asset not found"))?;

    let wrapped = text::wrap(&q.text, 22);
    let white = Rgba([255, 255, 255, 255]);
    let black = Rgba([0, 0, 0, 255]);
    let style = text::TextStyle::new(&font, 32.0).color(white).stroke(2, black);

    text::draw(&mut img, &state.http, &state.emojis, (60, 15), "guys i have bad news", &style).await;

    let img_w = img.width() as i32;
    let img_h = img.height() as i32;
    let (_, h) = text::measure(&font, 32.0, &wrapped, 1.0);
    let centered = style.align(text::Align::Center, img_w);
    text::draw(&mut img, &state.http, &state.emojis, (0, img_h - h - 20), &wrapped, &centered).await;

    let bytes = imaging::prepare_png(&img)?;
    Ok(([(header::CONTENT_TYPE, "image/png")], bytes))
}