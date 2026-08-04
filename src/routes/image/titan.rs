use crate::{imaging::{self, text}, response::{ApiError, ApiResult}, state::AppState, validate};
use crate::extract::Query;
use axum::{extract::State, http::header, response::IntoResponse};
use image::Rgba;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct TitanQuery { text1: String, text2: String }

/// GET /image/titan?text1=...&text2=... — attack on titan style meme with 2 captions.
pub async fn handler(State(state): State<AppState>, Query(q): Query<TitanQuery>) -> ApiResult<impl IntoResponse> {
    validate::len(&q.text1, 1, 150, "text1")?;
    validate::len(&q.text2, 1, 150, "text2")?;
    let font1 = state.fonts.fetch("GGSans", "Bold").map_err(|_| ApiError::internal("Font not found"))?;
    let mut image = state.images.fetch("titan").map_err(|_| ApiError::internal("Asset not found"))?;

    let white = Rgba([255, 255, 255, 255]);
    let black = Rgba([0, 0, 0, 255]);
    let style1 = text::TextStyle::new(&font1, 50.0).color(white).stroke(2, black);
    text::draw(&mut image, &state.http, &state.emojis, (360, 250), &text::wrap(&q.text1, 12), &style1).await;
    let style2 = text::TextStyle::new(&font1, 30.0).color(white).stroke(2, black);
    text::draw(&mut image, &state.http, &state.emojis, (160, 855), &text::wrap(&q.text2, 20), &style2).await;

    let bytes = imaging::prepare_png(&image)?;
    Ok(([(header::CONTENT_TYPE, "image/png")], bytes))
}