use crate::{imaging, response::{ApiError, ApiResult}, state::AppState};
use crate::extract::Query;
use axum::{extract::State, http::header, response::IntoResponse};
use image::imageops::FilterType;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct ImgQuery { image: String }

/// GET /image/simp?image=URL — overlays the "simp" stamp filter.
pub async fn handler(State(state): State<AppState>, Query(q): Query<ImgQuery>) -> ApiResult<impl IntoResponse> {
    let mut img = imaging::open_image(&state.http, &q.image, "image").await?;
    let overlay = state.images.fetch("simp").map_err(|_| ApiError::internal("Asset not found"))?;
    let overlay = image::imageops::resize(&overlay, img.width(), img.height(), FilterType::Lanczos3);
    imaging::paste(&mut img, &overlay, 0, 0);

    let bytes = imaging::prepare_png(&img)?;
    Ok(([(header::CONTENT_TYPE, "image/png")], bytes))
}
