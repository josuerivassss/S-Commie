use crate::{imaging, response::{ApiError, ApiResult}, state::AppState};
use crate::extract::Query;
use axum::{extract::State, http::header, response::IntoResponse};
use image::imageops::FilterType;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct ImgQuery { image: String }

/// GET /image/whoreallyare?image=URL.
pub async fn handler(State(state): State<AppState>, Query(q): Query<ImgQuery>) -> ApiResult<impl IntoResponse> {
    let avatar = imaging::open_image(&state.http, &q.image, "image").await?;
    let avatar = imaging::ellipse_mask(avatar);
    let avatar = image::imageops::resize(&avatar, 190, 190, FilterType::Lanczos3);
    let mut overlay = state.images.fetch("whoreally").map_err(|_| ApiError::internal("Asset not found"))?;
    imaging::paste(&mut overlay, &avatar, 68, 580);

    let bytes = imaging::prepare_png(&overlay)?;
    Ok(([(header::CONTENT_TYPE, "image/png")], bytes))
}
