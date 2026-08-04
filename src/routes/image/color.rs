use crate::{imaging, response::ApiResult, state::AppState, validate};
use crate::extract::Query;
use axum::{extract::State, http::header, response::IntoResponse};
use image::{Rgba, RgbaImage};
use imageproc::{drawing::draw_filled_rect_mut, rect::Rect};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct ColorQuery {
    code: String,
    #[serde(default = "default_size")]
    width: u32,
    #[serde(default = "default_size")]
    height: u32,
    #[serde(default = "default_true")]
    #[serde(rename = "showCode")]
    show_code: bool,
    #[serde(default = "default_radius")]
    radius: i32,
}
fn default_size() -> u32 { 512 }
fn default_true() -> bool { true }
fn default_radius() -> i32 { 15 }

/// GET /image/color?code=HEX&width=&height=&showCode=&radius= — solid color swatch.
pub async fn handler(State(state): State<AppState>, Query(q): Query<ColorQuery>) -> ApiResult<impl IntoResponse> {
    validate::hex_color(&q.code, "code")?;
    validate::range(q.width, 15, 1024, "width")?;
    validate::range(q.height, 15, 1024, "height")?;
    validate::range(q.radius, 0, 150, "radius")?;

    let color = imaging::parse_hex(&q.code);
    let mut img = RgbaImage::from_pixel(q.width, q.height, Rgba([0, 0, 0, 0]));
    // rounded rect approximated via a plain filled rect + corner masking (radius kept modest by validation)
    draw_filled_rect_mut(&mut img, Rect::at(0, 0).of_size(q.width, q.height), color);
    let mut img = imaging::rounded_mask(img, q.radius);

    if q.show_code {
        let font = state.fonts.fetch("GGSans", "Bold").or_else(|_| state.fonts.fetch("GGSans", "Regular"))
            .map_err(|_| crate::response::ApiError::internal("Font GGSans not found"))?;
        let text = format!("#{}", q.code);
        let style = imaging::text::TextStyle::new(&font, 45.0).color(Rgba([255, 255, 255, 255]));
        imaging::text::draw(&mut img, &state.http, &state.emojis, (25, q.height as i32 - 80), &text, &style).await;
    }

    let bytes = imaging::prepare_png(&img)?;
    Ok(([(header::CONTENT_TYPE, "image/png")], bytes))
}