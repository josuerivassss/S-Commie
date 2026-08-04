use crate::{imaging, response::{ApiError, ApiResult}, state::AppState};
use crate::extract::Query;
use axum::{extract::State, http::header, response::IntoResponse};
use image::imageops::FilterType;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct ImgQuery { image: String }

/// GET /image/delete?image=URL — "trash" meme.
pub async fn handler(State(state): State<AppState>, Query(q): Query<ImgQuery>) -> ApiResult<impl IntoResponse> {
    let avatar = imaging::open_image(&state.http, &q.image, "image").await?;
    let avatar = image::imageops::resize(&avatar, 180, 180, FilterType::Lanczos3);
    let mut base = state.images.fetch("delete").map_err(|_| ApiError::internal("Asset not found"))?;
    imaging::paste(&mut base, &avatar, 135, 135);

    let bytes = imaging::prepare_png(&base)?;
    Ok(([(header::CONTENT_TYPE, "image/png")], bytes))
}
