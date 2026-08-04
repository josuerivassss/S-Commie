use crate::{imaging, response::ApiResult, state::AppState, validate};
use crate::extract::Query;
use axum::{extract::State, http::header, response::IntoResponse};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct DeepfryQuery {
    image: String,
    #[serde(default = "default_amount")]
    amount: i32,
}
fn default_amount() -> i32 { 2 }

/// GET /image/deepfry?image=URL&amount=1..10 — cranks contrast around the mid-gray point.
pub async fn handler(State(state): State<AppState>, Query(q): Query<DeepfryQuery>) -> ApiResult<impl IntoResponse> {
    validate::range(q.amount, 1, 10, "amount")?;
    let mut img = imaging::open_image(&state.http, &q.image, "image").await?;
    let factor = q.amount as f32;
    for p in img.pixels_mut() {
        for c in 0..3 {
            let v = (p.0[c] as f32 - 128.0) * factor + 128.0;
            p.0[c] = v.clamp(0.0, 255.0) as u8;
        }
    }
    let bytes = imaging::prepare_png(&img)?;
    Ok(([(header::CONTENT_TYPE, "image/png")], bytes))
}
