use crate::{imaging, response::{ApiError, ApiResult}, state::AppState};
use crate::extract::Query;
use axum::{extract::State, http::header, response::IntoResponse};
use image::{imageops::FilterType, Rgba, RgbaImage};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct ShipQuery {
    #[serde(rename = "image1")]
    image: String,
    image2: String,
    #[serde(default = "default_style")]
    style: String,
    background: Option<String>,
    #[serde(default = "default_true")]
    blur: bool,
}
fn default_style() -> String { "normal".to_string() }
fn default_true() -> bool { true }

/// GET /image/ship?image1=&image2=&style=normal|fire|broken&background=&blur= — "ship" compatibility card.
pub async fn handler(State(state): State<AppState>, Query(q): Query<ShipQuery>) -> ApiResult<impl IntoResponse> {
    if !["normal", "fire", "broken"].contains(&q.style.as_str()) {
        return Err(ApiError::validation("style must be one of: normal, fire, broken", "style"));
    }

    let radius = if q.blur { 25.0 } else { 0.0 };
    let mut base = RgbaImage::from_pixel(1250, 500, Rgba([0, 0, 0, 0]));

    let img1 = imaging::open_image(&state.http, &q.image, "image1").await?;
    let img2 = imaging::open_image(&state.http, &q.image2, "image2").await?;
    let img1 = image::imageops::resize(&img1, 650, 650, FilterType::Lanczos3);
    let img2 = image::imageops::resize(&img2, 650, 650, FilterType::Lanczos3);

    if let Some(bg_url) = &q.background {
        let bg = imaging::open_image(&state.http, bg_url, "background").await?;
        let bg = image::imageops::resize(&bg, 1250, 500, FilterType::Lanczos3);
        let bg = image::imageops::blur(&bg, radius);
        imaging::paste(&mut base, &bg, 0, 0);
    } else {
        let blurred1 = image::imageops::blur(&img1, radius);
        let blurred2 = image::imageops::blur(&img2, radius);
        let half_w = base.width() as i64 / 2;
        imaging::paste(&mut base, &blurred1, -10, -50);
        imaging::paste(&mut base, &blurred2, half_w, -50);
    }

    let card1 = image::imageops::resize(&imaging::rounded_mask(img1, 20), 372, 372, FilterType::Lanczos3);
    let card2 = image::imageops::resize(&imaging::rounded_mask(img2, 20), 372, 372, FilterType::Lanczos3);
    imaging::paste(&mut base, &card1, 60, 70);
    imaging::paste(&mut base, &card2, 820, 70);

    let heart = state.images.fetch(&format!("heart_{}", q.style)).map_err(|_| ApiError::internal("Asset not found"))?;
    let heart = image::imageops::resize(&heart, 323, 323, FilterType::Lanczos3);
    imaging::paste(&mut base, &heart, 465, 95);

    let bytes = imaging::prepare_png(&base)?;
    Ok(([(header::CONTENT_TYPE, "image/png")], bytes))
}
