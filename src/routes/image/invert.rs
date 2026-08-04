use crate::{imaging, response::ApiResult, state::AppState};
use crate::extract::Query;
use axum::{extract::State, http::header, response::IntoResponse};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct ImgQuery { image: String }

/// GET /image/invert?image=URL — flips R/G/B, keeps original alpha untouched.
pub async fn handler(State(state): State<AppState>, Query(q): Query<ImgQuery>) -> ApiResult<impl IntoResponse> {
    let mut img = imaging::open_image(&state.http, &q.image, "image").await?;
    for p in img.pixels_mut() {
        p.0[0] = 255 - p.0[0];
        p.0[1] = 255 - p.0[1];
        p.0[2] = 255 - p.0[2];
    }
    let bytes = imaging::prepare_png(&img)?;
    Ok(([(header::CONTENT_TYPE, "image/png")], bytes))
}
