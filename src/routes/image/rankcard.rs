use crate::{imaging::{self, text}, response::{ApiError, ApiResult}, state::AppState, validate};
use crate::extract::Query;
use axum::{extract::State, http::header, response::IntoResponse};
use image::{imageops::FilterType, Rgba, RgbaImage};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct RankCardQuery {
    image: String,
    username: String,
    xp: f64,
    total: f64,
    #[serde(default)]
    level: u32,
    #[serde(default)]
    rank: u32,
    #[serde(default = "default_color")]
    color: String,
    background: Option<String>,
    #[serde(default = "default_true")]
    blur: bool,
}
fn default_color() -> String { "5865F2".to_string() }
fn default_true() -> bool { true }

/// Maps xp/total onto the 425px-wide progress bar, clamped to [0, 425].
fn bar_width(xp: f64, total: f64) -> f64 {
    let progress = (xp / total) * 425.0;
    if !progress.is_finite() || progress < 0.0 {
        1.0
    } else {
        progress.min(425.0)
    }
}

/// GET /image/rankcard — level/XP profile card.
pub async fn handler(State(state): State<AppState>, Query(q): Query<RankCardQuery>) -> ApiResult<impl IntoResponse> {
    validate::hex_color(&q.color, "color")?;
    validate::range(q.xp, 0.0, 1_000_000.0, "xp")?;
    validate::range(q.total, 0.0, 1_000_000.0, "total")?;
    if q.xp > q.total {
        return Err(ApiError::validation("XP value cannot be greater than TOTAL value", "xp"));
    }

    let avatar = imaging::open_image(&state.http, &q.image, "image").await?;
    let (w, h) = (750u32, 256u32);
    let mut base = RgbaImage::from_pixel(w, h, Rgba([0, 0, 0, 0]));

    if let Some(bg_url) = &q.background {
        let bg = imaging::open_image(&state.http, bg_url, "background").await?;
        let bg = image::imageops::resize(&bg, w, h, FilterType::Lanczos3);
        let bg = image::imageops::blur(&bg, if q.blur { 20.0 } else { 0.0 });
        imaging::paste(&mut base, &bg, 0, 0);
    } else {
        let bg = image::imageops::resize(&avatar, 750, 750, FilterType::Lanczos3);
        let bg = image::imageops::blur(&bg, if q.blur { 20.0 } else { 0.0 });
        imaging::paste(&mut base, &bg, (w as i64 - 750) / 2, (h as i64 - 750) / 2);
    }

    // dark translucent panel so the text stays legible over any background
    let panel = imaging::rounded_rect_image(w, h, 30, Rgba([0, 0, 0, 180]));
    imaging::paste(&mut base, &panel, 0, 0);

    let avatar = imaging::rounded_mask(avatar, 130);
    let avatar = image::imageops::resize(&avatar, 225, 225, FilterType::Lanczos3);
    let midpoint = 0 + (h as i64 - 225) / 2;
    imaging::paste(&mut base, &avatar, 30, midpoint);

    let font_big = state.fonts.fetch("FranklinGothicDemi", "Regular").map_err(|_| ApiError::internal("Font not found"))?;
    let font_small = state.fonts.fetch("GGSans", "Medium").map_err(|_| ApiError::internal("Font not found"))?;

    let style_big = text::TextStyle::new(&font_big, 44.0).color(Rgba([255, 255, 255, 255]));
    text::draw(&mut base, &state.http, &state.emojis, (285, 40), &q.username, &style_big).await;
    let style_small = text::TextStyle::new(&font_small, 16.0).color(Rgba([218, 218, 218, 255]));
    text::draw(&mut base, &state.http, &state.emojis, (287, 90), &format!("Level: {}    Rank: #{}", q.level, q.rank), &style_small).await;
    text::draw(&mut base, &state.http, &state.emojis, (290, 160), &format!("XP: {}   /   {}", q.xp, q.total), &style_small).await;

    let main_color = imaging::parse_hex(&q.color);
    let track_color = imaging::darken(main_color, 0.50);
    let track = imaging::rounded_rect_image(425, 40, 15, track_color);
    imaging::paste(&mut base, &track, 285, 185);

    if q.xp >= 1.0 {
        let fill_w = bar_width(q.xp, q.total).ceil().max(8.0) as u32;
        // outline effect: darker rect first (slightly larger), colored fill on top
        let outline = imaging::rounded_rect_image(fill_w + 6, 34 + 6, 18, track_color);
        imaging::paste(&mut base, &outline, 285, 185);
        let fill = imaging::rounded_rect_image(fill_w, 34, 15, main_color);
        imaging::paste(&mut base, &fill, 288, 188);
    }

    let base = imaging::rounded_mask(base, 30);
    let bytes = imaging::prepare_png(&base)?;
    Ok(([(header::CONTENT_TYPE, "image/png")], bytes))
}