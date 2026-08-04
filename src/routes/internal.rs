use crate::{response::{ApiError, ApiOk, ApiResult}, state::AppState};
use axum::{extract::Query, routing::get, Router};
use serde::Deserialize;
use serde_json::json;

const ROUTES: &[(&str, &str)] = &[
    ("/json/binary", "Encode or decode a binary text"),
    ("/json/8ball", "Get a random 8ball response"),
    ("/json/animegifs", "Get a random anime gif"),
    ("/json/calendar", "Check the calendar of a month"),
    ("/json/ocr", "Extract text from an image (OCR)"),
    ("/image/grayscale", "Apply a grayscale filter to your image"),
    ("/image/invert", "Apply an invert filter to your image"),
    ("/image/mirror", "Apply a mirror effect to your image"),
    ("/image/blur", "Apply a blur filter to your image"),
    ("/image/deepfry", "Apply a deepfry filter to your image"),
    ("/image/pixel", "Apply a pixel filter to your image"),
    ("/image/circle", "Apply a circle cut to your image"),
    ("/image/color", "Make an image of the color you provided"),
    ("/image/badnews", "Make a bad news meme with your own text"),
    ("/image/discordjs", "Make a discordjs-like logo with your own image"),
    ("/image/supreme", "Make a supreme-like logo with your own image"),
    ("/image/santa", "Make a santa meme with your own text"),
    ("/image/facts", "Make a facts meme with your own text"),
    ("/image/sonic", "Make a sonic meme with your own text"),
    ("/image/titan", "Make a titan meme with your own text"),
    ("/image/twoways", "Make a two ways meme with your own text"),
    ("/image/thisis", "Make a playlist image using your image and text"),
    ("/image/beautiful", "Make a beautiful meme using your own image"),
    ("/image/communist", "Apply a communist filter to your image"),
    ("/image/rainbow", "Apply a rainbow filter to your image"),
    ("/image/simp", "Apply a simp filter to your image"),
    ("/image/sus", "Make a sus (among us) image using your own image"),
    ("/image/mad", "Make a mad image using your own image"),
    ("/image/delete", "Make a delete trash meme using your own image"),
    ("/image/whoreallyare", "Make a whoreallyare image using your own image"),
    ("/image/ship", "Make a ship image of two images"),
    ("/image/rankcard", "Make a rankcard using your own data"),
    ("/image/walletcard", "Make a wallet card using your own data"),
    ("/image/welcomecard", "Make a welcomecard using your own data"),
];

#[derive(Deserialize)]
pub struct HelpQuery {
    query: String,
}

async fn root() -> ApiResult {
    Ok(ApiOk::new(json!({
        "name": "SCOMMIE",
        "version": env!("CARGO_PKG_VERSION"),
        "routes": ROUTES.iter().map(|(p, d)| json!({"path": p, "description": d})).collect::<Vec<_>>(),
    })))
}

/// GET /help?query=... — fuzzy-ish search over the route list (path or description contains the query).
async fn help(Query(q): Query<HelpQuery>) -> ApiResult {
    let needle = q.query.to_lowercase();
    let matches: Vec<_> = ROUTES
        .iter()
        .filter(|(path, desc)| path.to_lowercase().contains(&needle) || desc.to_lowercase().contains(&needle))
        .map(|(p, d)| json!({"path": p, "description": d}))
        .collect();

    if matches.is_empty() {
        return Err(ApiError::not_found());
    }
    Ok(ApiOk::new(matches))
}

pub fn router() -> Router<AppState> {
    Router::new().route("/", get(root)).route("/help", get(help))
}
