use crate::{imaging, response::ApiResult, state::AppState, validate};
use crate::extract::Query;
use axum::{extract::State, http::header, response::IntoResponse};
use image::imageops::FilterType;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct PixelQuery {
    image: String,
    #[serde(default = "default_amount")]
    amount: u32,
}
fn default_amount() -> u32 { 5 }

/// GET /image/pixel?image=URL&amount=1..100 — downscale then upscale with nearest-neighbor.
pub async fn handler(State(state): State<AppState>, Query(q): Query<PixelQuery>) -> ApiResult<impl IntoResponse> {
    validate::range(q.amount, 1, 100, "amount")?;
    let img = imaging::open_image(&state.http, &q.image, "image").await?;
    let (w, h) = img.dimensions();
    let (sw, sh) = ((w / q.amount).max(1), (h / q.amount).max(1));
    let small = image::imageops::resize(&img, sw, sh, FilterType::Nearest);
    let big = image::imageops::resize(&small, w, h, FilterType::Nearest);
    let bytes = imaging::prepare_png(&big)?;
    Ok(([(header::CONTENT_TYPE, "image/png")], bytes))
}
