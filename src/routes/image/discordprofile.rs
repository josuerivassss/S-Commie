use crate::{imaging::{self, text}, response::{ApiError, ApiResult}, state::AppState, validate};
use crate::extract::Query;
use axum::{extract::State, http::header, response::IntoResponse};
use image::{imageops::FilterType, Rgba, RgbaImage};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct DiscordProfileQuery {
    avatar: String,
    banner: Option<String>,
    #[serde(default = "default_status")]
    status: String,
    text: String,
    emoji: Option<String>,
    bar: Option<String>,
}
fn default_status() -> String { "online".to_string() }

const CANVAS_W: u32 = 900;
const CANVAS_H: u32 = 360;
const BANNER_H: u32 = 240;
const AVATAR_RADIUS: i32 = 70;
const AVATAR_CENTER: (i32, i32) = (140, 240);
const DEFAULT_BAR_HEX: &str = "1e1f22";

/// GET /image/discordprofile?avatar=&banner=&status=&text=&emoji=&bar=
/// Renders a customizable Discord-style profile card for previewing
/// avatars/banners without setting them on a real account.
pub async fn handler(State(state): State<AppState>, Query(q): Query<DiscordProfileQuery>) -> ApiResult<impl IntoResponse> {
    validate::len(&q.text, 1, 100, "text")?;
    if let Some(bar) = &q.bar {
        validate::hex_color(bar, "bar")?;
    }

    let bar_color = q.bar.as_deref().map(imaging::parse_hex).unwrap_or_else(|| imaging::parse_hex(DEFAULT_BAR_HEX));

    let avatar = imaging::open_image(&state.http, &q.avatar, "avatar").await?;
    let avatar = image::imageops::resize(&avatar, (AVATAR_RADIUS * 2) as u32, (AVATAR_RADIUS * 2) as u32, FilterType::Lanczos3);
    let avatar_circle = imaging::ellipse_mask(avatar.clone());

    let mut canvas = RgbaImage::from_pixel(CANVAS_W, CANVAS_H, bar_color);

    let banner = resolve_banner(&state, &q.banner, &avatar).await?;
    imaging::paste(&mut canvas, &banner, 0, 0);

    let ring_radius = AVATAR_RADIUS + 8;
    imageproc::drawing::draw_filled_ellipse_mut(&mut canvas, AVATAR_CENTER, ring_radius, ring_radius, bar_color);
    imaging::paste(&mut canvas, &avatar_circle, (AVATAR_CENTER.0 - AVATAR_RADIUS) as i64, (AVATAR_CENTER.1 - AVATAR_RADIUS) as i64);

    let status = imaging::PresenceStatus::from_str(&q.status);
    let badge_center = (AVATAR_CENTER.0 + (AVATAR_RADIUS as f32 * 0.72) as i32, AVATAR_CENTER.1 + (AVATAR_RADIUS as f32 * 0.72) as i32);
    imaging::draw_status_badge(&mut canvas, status, badge_center, 20, bar_color);

    draw_status_pill(&mut canvas, &state, &q).await?;

    let bytes = imaging::prepare_png(&canvas)?;
    Ok(([(header::CONTENT_TYPE, "image/png")], bytes))
}

fn looks_like_hex(value: &str) -> bool {
    let c = value.trim_start_matches('#');
    (c.len() == 3 || c.len() == 6) && c.chars().all(|ch| ch.is_ascii_hexdigit())
}

async fn resolve_banner(state: &AppState, banner: &Option<String>, avatar: &RgbaImage) -> Result<RgbaImage, ApiError> {
    match banner {
        Some(value) if looks_like_hex(value) => Ok(RgbaImage::from_pixel(CANVAS_W, BANNER_H, imaging::parse_hex(value))),
        Some(url) => {
            let img = imaging::open_image(&state.http, url, "banner").await?;
            Ok(image::imageops::resize(&img, CANVAS_W, BANNER_H, FilterType::Lanczos3))
        }
        None => {
            let dominant = imaging::dominant_colors(avatar, 1);
            let color = dominant.first().copied().unwrap_or([54, 57, 63]);
            Ok(RgbaImage::from_pixel(CANVAS_W, BANNER_H, Rgba([color[0], color[1], color[2], 255])))
        }
    }
}

/// `emoji` accepts a direct image URL (custom Discord emoji, user pastes the
/// CDN link themselves) or a literal unicode emoji (resolved through the
/// existing Twemoji-backed EmojiCache).
async fn resolve_icon(state: &AppState, emoji: &str, size: u32) -> Option<RgbaImage> {
    if emoji.starts_with("http://") || emoji.starts_with("https://") {
        let img = imaging::open_image(&state.http, emoji, "emoji").await.ok()?;
        Some(image::imageops::resize(&img, size, size, FilterType::Lanczos3))
    } else {
        let ch = emoji.chars().next()?;
        let glyph = state.emojis.get(&state.http, ch, size).await?;
        Some((*glyph).clone())
    }
}

async fn draw_status_pill(canvas: &mut RgbaImage, state: &AppState, q: &DiscordProfileQuery) -> Result<(), ApiError> {
    let font = state.fonts.fetch("GGSans", "Medium").map_err(|_| ApiError::internal("Font GGSans not found"))?;
    let text_scale = 26.0;
    let (text_w, _) = text::measure(&font, text_scale, &q.text, 1.0);

    let icon_size: u32 = 30;
    let icon = match &q.emoji {
        Some(value) => resolve_icon(state, value, icon_size).await,
        None => None,
    };

    let padding = 18;
    let gap = if icon.is_some() { 12 } else { 0 };
    let icon_space = if icon.is_some() { icon_size as i32 } else { 0 };

    let content_w = icon_space + gap + text_w;
    let max_w = CANVAS_W as i32 - (AVATAR_CENTER.0 + AVATAR_RADIUS + 30) - 30;
    let pill_w = (content_w + padding * 2).clamp(60, max_w);
    let pill_h = 54;
    let bar_area_h = (CANVAS_H - BANNER_H) as i32;
    let pill_x = AVATAR_CENTER.0 + AVATAR_RADIUS + 30;
    let pill_y = BANNER_H as i32 + (bar_area_h - pill_h) / 2;

    let pill = imaging::rounded_rect_image(pill_w as u32, pill_h as u32, pill_h / 2, Rgba([15, 15, 20, 235]));
    imaging::paste(canvas, &pill, pill_x as i64, pill_y as i64);

    let mut cursor_x = pill_x + padding;
    let content_y = pill_y + pill_h / 2;

    if let Some(icon_img) = icon {
        let icon_y = content_y - icon_size as i32 / 2;
        imaging::paste(canvas, &icon_img, cursor_x as i64, icon_y as i64);
        cursor_x += icon_size as i32 + gap;
    }

    let text_y = content_y - (text_scale as i32) / 2;
    let style = text::TextStyle::new(&font, text_scale).color(Rgba([255, 255, 255, 255]));
    text::draw(canvas, &state.http, &state.emojis, (cursor_x, text_y), &q.text, &style).await;

    Ok(())
}