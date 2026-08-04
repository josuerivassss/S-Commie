use crate::response::ApiError;
use axum::http::StatusCode;
use image::{codecs::png::PngEncoder, ColorType, ImageEncoder, RgbaImage};

/// Downloads a remote image (or opens a local one when prefixed with "path:")
/// and decodes it into an RGBA buffer. `field` is only used to build a useful
/// validation error (mirrors FastAPI's `loc` field).
pub async fn open_image(client: &reqwest::Client, source: &str, field: &str) -> Result<RgbaImage, ApiError> {
    if source.trim().is_empty() {
        return Err(ApiError::validation("Missing image URL", field));
    }

    if let Some(local_path) = source.strip_prefix("path:") {
        return image::open(local_path)
            .map(|img| img.to_rgba8())
            .map_err(|_| ApiError::validation("Invalid image URL provided", field));
    }

    let bytes = client
        .get(source)
        .send()
        .await
        .and_then(|r| r.error_for_status())
        .map_err(|_| ApiError::validation("Invalid image URL provided", field))?
        .bytes()
        .await
        .map_err(|_| ApiError::validation("Invalid image URL provided", field))?;

    image::load_from_memory(&bytes)
        .map(|img| img.to_rgba8())
        .map_err(|_| ApiError::validation("Invalid image URL provided", field))
}

/// Encodes an RGBA image as PNG bytes, ready to be returned in the HTTP response.
/// Encodes straight from the raw buffer (no extra clone/allocation via DynamicImage).
pub fn prepare_png(img: &RgbaImage) -> Result<Vec<u8>, ApiError> {
    let mut buf = Vec::new();
    PngEncoder::new(&mut buf)
        .write_image(img.as_raw(), img.width(), img.height(), ColorType::Rgba8)
        .map_err(|_| ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, "Failed to encode image"))?;
    Ok(buf)
}
