use crate::{imaging, response::{ApiResult}, state::AppState, validate};
use crate::extract::Query;
use axum::{extract::State, http::header, response::IntoResponse};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct BlurQuery {
    image: String,
    #[serde(default = "default_radius")]
    radius: i32,
}
fn default_radius() -> i32 { 2 }

/// GET /image/blur?image=URL&radius=1..10
pub async fn handler(State(state): State<AppState>, Query(q): Query<BlurQuery>) -> ApiResult<impl IntoResponse> {
    validate::range(q.radius, 1, 10, "radius")?;
    let img = imaging::open_image(&state.http, &q.image, "image").await?;
    let blurred = image::imageops::blur(&img, q.radius as f32);
    let bytes = imaging::prepare_png(&blurred)?;
    Ok(([(header::CONTENT_TYPE, "image/png")], bytes))
}
