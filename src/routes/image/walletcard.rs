use crate::{imaging::{self, text}, response::{ApiError, ApiResult}, state::AppState, validate};
use crate::extract::Query;
use axum::{extract::State, http::header, response::IntoResponse};
use image::{imageops::FilterType, Rgba, RgbaImage};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct WalletCardQuery {
    image: String,
    username: String,
    #[serde(default)]
    wallet: f64,
    #[serde(default)]
    bank: f64,
    background: Option<String>,
    #[serde(default = "default_footer")]
    footer: String,
    #[serde(default = "default_true")]
    blur: bool,
}
fn default_footer() -> String { "Commie".to_string() }
fn default_true() -> bool { true }

/// GET /image/walletcard — economy/balance card.
pub async fn handler(State(state): State<AppState>, Query(q): Query<WalletCardQuery>) -> ApiResult<impl IntoResponse> {
    validate::len(&q.username, 1, 200, "username")?;
    let avatar = imaging::open_image(&state.http, &q.image, "image").await?;
    let overlay = state.images.fetch("wallet").map_err(|_| ApiError::internal("Asset not found"))?;
    let (w, h) = (overlay.width(), overlay.height());
    let mut base = RgbaImage::from_pixel(w, h, Rgba([0, 0, 0, 0]));

    if let Some(bg_url) = &q.background {
        let bg = imaging::open_image(&state.http, bg_url, "background").await?;
        let bg = image::imageops::resize(&bg, w, h, FilterType::Lanczos3);
        let bg = image::imageops::blur(&bg, if q.blur { 10.0 } else { 0.0 });
        imaging::paste(&mut base, &bg, 0, 0);
    } else {
        let colors = imaging::dominant_colors(&avatar, 2);
        imaging::draw_gradient(&mut base, ((-2, -2), (520, 285)), &colors, true);
    }

    let avatar = imaging::ellipse_mask(avatar);
    let avatar = image::imageops::resize(&avatar, 42, 42, FilterType::Lanczos3);
    imaging::paste(&mut base, &avatar, 50, 47);
    imaging::paste(&mut base, &overlay, 0, 0);

    let font_medium = state.fonts.fetch("GGSans", "Medium").map_err(|_| ApiError::internal("Font not found"))?;
    let font_bold = state.fonts.fetch("GGSans", "Bold").map_err(|_| ApiError::internal("Font not found"))?;
    let font_normal = state.fonts.fetch("GGSans", "Regular").map_err(|_| ApiError::internal("Font not found"))?;
    let white = Rgba([255, 255, 255, 255]);

    let style_medium = text::TextStyle::new(&font_medium, 17.0).color(white);
    text::draw(&mut base, &state.http, &state.emojis, (105, 56), &q.username, &style_medium).await;

    let footer_style = text::TextStyle::new(&font_bold, 16.0).color(Rgba([218, 218, 218, 255])).align(text::Align::Center, w as i32);
    text::draw(&mut base, &state.http, &state.emojis, (0, 240), &q.footer, &footer_style).await;

    let style_normal = text::TextStyle::new(&font_normal, 19.0).color(white);
    text::draw(&mut base, &state.http, &state.emojis, (115, 111), &format!("{:.2}", q.wallet), &style_normal).await;
    text::draw(&mut base, &state.http, &state.emojis, (115, 158), &format!("{:.2}", q.bank), &style_normal).await;
    text::draw(&mut base, &state.http, &state.emojis, (115, 204), &format!("{:.2}", q.wallet + q.bank), &style_normal).await;

    let bytes = imaging::prepare_png(&base)?;
    Ok(([(header::CONTENT_TYPE, "image/png")], bytes))
}