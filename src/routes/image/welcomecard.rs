use crate::{imaging::{self, text}, response::{ApiError, ApiResult}, state::AppState, validate};
use crate::extract::Query;
use axum::{extract::State, http::header, response::IntoResponse};
use image::{imageops::FilterType, Rgba, RgbaImage};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct WelcomeCardQuery {
    image: String,
    #[serde(default = "default_title")]
    title: String,
    #[serde(default = "default_subtitle")]
    subtitle: String,
    background: Option<String>,
    #[serde(default = "default_true")]
    blur: bool,
}
fn default_title() -> String { "Welcome".to_string() }
fn default_subtitle() -> String { "Enjoy your stay!".to_string() }
fn default_true() -> bool { true }

/// GET /image/welcomecard — server welcome banner.
pub async fn handler(State(state): State<AppState>, Query(q): Query<WelcomeCardQuery>) -> ApiResult<impl IntoResponse> {
    validate::len(&q.title, 1, 80, "title")?;
    validate::len(&q.subtitle, 1, 100, "subtitle")?;

    let avatar = imaging::open_image(&state.http, &q.image, "image").await?;
    let (w, h) = (1024u32, 500u32);
    let mut base = RgbaImage::from_pixel(w, h, Rgba([0, 0, 0, 255]));

    if let Some(bg_url) = &q.background {
        let bg = imaging::open_image(&state.http, bg_url, "background").await?;
        let bg = image::imageops::resize(&bg, w, h, FilterType::Lanczos3);
        let bg = image::imageops::blur(&bg, if q.blur { 10.0 } else { 0.0 });
        imaging::paste(&mut base, &bg, 0, 0);
    } else {
        let colors = imaging::dominant_colors(&avatar, 2);
        imaging::draw_gradient(&mut base, ((0, 0), (w, h)), &colors, true);
    }

    // translucent rounded panel so the avatar/title/subtitle stay legible
    let panel = imaging::rounded_rect_image(w - 90, h - 90, 20, Rgba([0, 0, 0, 200]));
    imaging::paste(&mut base, &panel, 45, 45);

    let avatar = imaging::ellipse_mask(avatar);
    let avatar = image::imageops::resize(&avatar, 220, 220, FilterType::Lanczos3);
    imaging::paste(&mut base, &avatar, (w as i64 - 220) / 2, 65);

    let font_title = state.fonts.fetch("SuperCorn", "Regular").map_err(|_| ApiError::internal("Font not found"))?;
    let font_sub = state.fonts.fetch("FranklinGothicDemi", "Regular").map_err(|_| ApiError::internal("Font not found"))?;
    let white = Rgba([255, 255, 255, 255]);

    let title_style = text::TextStyle::new(&font_title, 65.0).color(white).align(text::Align::Center, w as i32);
    text::draw(&mut base, &state.http, &state.emojis, (0, 295), &q.title, &title_style).await;

    let subtitle_style = text::TextStyle::new(&font_sub, 33.0).color(white).align(text::Align::Center, w as i32);
    text::draw(&mut base, &state.http, &state.emojis, (0, 360), &q.subtitle, &subtitle_style).await;

    let bytes = imaging::prepare_png(&base)?;
    Ok(([(header::CONTENT_TYPE, "image/png")], bytes))
}