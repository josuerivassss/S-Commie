use crate::{imaging, response::ApiResult, state::AppState};
use crate::extract::Query;
use axum::{extract::State, http::header, response::IntoResponse};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct ImgQuery { image: String }

/// GET /image/mirror?image=URL — horizontal flip.
pub async fn handler(State(state): State<AppState>, Query(q): Query<ImgQuery>) -> ApiResult<impl IntoResponse> {
    let img = imaging::open_image(&state.http, &q.image, "image").await?;
    let flipped = image::imageops::flip_horizontal(&img);
    let bytes = imaging::prepare_png(&flipped)?;
    Ok(([(header::CONTENT_TYPE, "image/png")], bytes))
}
