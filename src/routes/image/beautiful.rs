use crate::{imaging, response::{ApiError, ApiResult}, state::AppState};
use crate::extract::Query;
use axum::{extract::State, http::header, response::IntoResponse};
use image::imageops::FilterType;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct ImgQuery { image: String }

/// GET /image/beautiful?image=URL — "beautiful" meme with the avatar pasted twice.
pub async fn handler(State(state): State<AppState>, Query(q): Query<ImgQuery>) -> ApiResult<impl IntoResponse> {
    let avatar = imaging::open_image(&state.http, &q.image, "image").await?;
    let avatar = image::imageops::resize(&avatar, 104, 106, FilterType::Lanczos3);
    let mut base = state.images.fetch("beautiful").map_err(|_| ApiError::internal("Asset not found"))?;

    imaging::paste(&mut base, &avatar, 252, 25);
    imaging::paste(&mut base, &avatar, 252, 225);

    let bytes = imaging::prepare_png(&base)?;
    Ok(([(header::CONTENT_TYPE, "image/png")], bytes))
}
