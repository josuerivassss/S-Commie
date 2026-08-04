use crate::{imaging, response::ApiResult, state::AppState};
use crate::extract::Query;
use axum::{extract::State, http::header, response::IntoResponse};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct ImgQuery { image: String }

/// GET /image/grayscale?image=URL
pub async fn handler(State(state): State<AppState>, Query(q): Query<ImgQuery>) -> ApiResult<impl IntoResponse> {
    let img = imaging::open_image(&state.http, &q.image, "image").await?;
    let gray = image::imageops::grayscale(&img);
    let rgba = image::DynamicImage::ImageLuma8(gray).to_rgba8();
    let bytes = imaging::prepare_png(&rgba)?;
    Ok(([(header::CONTENT_TYPE, "image/png")], bytes))
}
