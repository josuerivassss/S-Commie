use crate::{imaging, response::{ApiError, ApiResult}, state::AppState};
use crate::extract::Query;
use axum::{extract::State, http::header, response::IntoResponse};
use image::{imageops::FilterType, Rgba, RgbaImage};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct ImgQuery { image: String }

/// GET /image/sus?image=URL — Among Us "sus" meme.
pub async fn handler(State(state): State<AppState>, Query(q): Query<ImgQuery>) -> ApiResult<impl IntoResponse> {
    let avatar = imaging::open_image(&state.http, &q.image, "image").await?;
    let avatar = image::imageops::resize(&avatar, 277, 277, FilterType::Lanczos3);
    let overlay = state.images.fetch("sus").map_err(|_| ApiError::internal("Asset not found"))?;

    let mut base = RgbaImage::from_pixel(512, 512, Rgba([0, 0, 0, 0]));
    imaging::paste(&mut base, &avatar, 210, 377 / 2);
    imaging::paste(&mut base, &overlay, 0, 0);

    let bytes = imaging::prepare_png(&base)?;
    Ok(([(header::CONTENT_TYPE, "image/png")], bytes))
}
