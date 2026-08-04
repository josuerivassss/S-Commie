use crate::extract::Query;
use crate::{imaging, response::{ApiError, ApiOk, ApiResult}, state::AppState, validate};
use axum::extract::State;
use image::imageops::FilterType;
use ocrs::ImageSource;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct OcrQuery {
    image: String,
}

#[derive(Serialize)]
struct OcrOut {
    text: String,
    lines: Vec<String>,
}

/// Images larger than this (longest side, px) are downscaled before inference.
/// OCR cost scales with pixel count, so this keeps CPU/RAM usage predictable
/// regardless of how large the source image is.
const MAX_SIDE: u32 = 2000;

/// GET /json/ocr?image=URL — detects and recognizes text in an image.
pub async fn handler(State(state): State<AppState>, Query(q): Query<OcrQuery>) -> ApiResult {
    validate::len(&q.image, 1, 2000, "image")?;

    let engine = state
        .ocr
        .clone()
        .ok_or_else(|| ApiError::internal("OCR engine not available (models not found on the server)"))?;

    let img = imaging::open_image(&state.http, &q.image, "image").await?;
    let mut rgb = image::DynamicImage::ImageRgba8(img).into_rgb8();

    let (w, h) = rgb.dimensions();
    if w.max(h) > MAX_SIDE {
        let scale = MAX_SIDE as f32 / w.max(h) as f32;
        rgb = image::imageops::resize(&rgb, (w as f32 * scale) as u32, (h as f32 * scale) as u32, FilterType::Triangle);
    }

    // Inference is synchronous/CPU-bound: run it on a blocking thread so it
    // never stalls the async runtime that's serving other requests.
    let lines = tokio::task::spawn_blocking(move || -> anyhow::Result<Vec<String>> {
        let source = ImageSource::from_bytes(rgb.as_raw(), rgb.dimensions())?;
        let input = engine.prepare_input(source)?;
        let word_rects = engine.detect_words(&input)?;
        let line_rects = engine.find_text_lines(&input, &word_rects);
        let line_texts = engine.recognize_text(&input, &line_rects)?;

        Ok(line_texts
            .into_iter()
            .flatten()
            // filters out likely spurious single-character detections
            .map(|l| l.to_string())
            .filter(|l| l.len() > 1)
            .collect())
    })
    .await
    .map_err(|_| ApiError::internal("OCR task panicked"))?
    .map_err(|_| ApiError::internal("OCR processing failed"))?;

    let text = lines.join("\n");
    Ok(ApiOk::new(OcrOut { text, lines }))
}