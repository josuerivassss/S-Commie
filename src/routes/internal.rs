use crate::{response::{ApiError, ApiOk, ApiResult}, state::AppState};
use axum::{extract::{Query, State, Path}, http::{HeaderMap, StatusCode}, routing::{get, post}, Router};
use serde::Deserialize;
use serde_json::json;

const ROUTES: &[(&str, &str)] = &[
    ("/json/binary", "Encode or decode a binary text"),
    ("/json/8ball", "Get a random 8ball response"),
    ("/json/animegifs", "Get a random anime gif"),
    ("/json/calendar", "Check the calendar of a month"),
    ("/json/ocr", "Extract text from an image (OCR)"),
    ("/json/translate", "Translate a text"),
    ("/json/weather", "Check the weather status in a specified location"),
    ("/json/imagesearch", "Search for images (DuckDuckGo, falls back to Bing)"),
    ("/image/grayscale", "Apply a grayscale filter to your image"),
    ("/image/invert", "Apply an invert filter to your image"),
    ("/image/mirror", "Apply a mirror effect to your image"),
    ("/image/blur", "Apply a blur filter to your image"),
    ("/image/deepfry", "Apply a deepfry filter to your image"),
    ("/image/pixel", "Apply a pixel filter to your image"),
    ("/image/circle", "Apply a circle cut to your image"),
    ("/image/color", "Make an image of the color you provided"),
    ("/image/badnews", "Make a bad news meme with your own text"),
    ("/image/supreme", "Make a supreme-like logo with your own image"),
    ("/image/sonic", "Make a sonic meme with your own text"),
    ("/image/titan", "Make a titan meme with your own text"),
    ("/image/twoways", "Make a two ways meme with your own text"),
    ("/image/thisis", "Make a playlist image using your image and text"),
    ("/image/communist", "Apply a communist filter to your image"),
    ("/image/rainbow", "Apply a rainbow filter to your image"),
    ("/image/sus", "Make a sus (among us) image using your own image"),
    ("/image/mad", "Make a mad image using your own image"),
    ("/image/caught", "Make a caught scooby-doo image using your own image"),
    ("/image/ship", "Make a ship image of two images"),
    ("/image/rankcard", "Make a rankcard using your own data"),
    ("/image/welcomecard", "Make a welcomecard using your own data"),
    ("/image/discordprofile", "Renders a customizable Discord-style profile card"),
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

async fn refresh_keys(State(state): State<AppState>, headers: HeaderMap) -> ApiResult {
    let expected = std::env::var("INTERNAL_ADMIN_SECRET").unwrap_or_default();
    let provided = headers.get("X-Internal-Secret").and_then(|v| v.to_str().ok()).unwrap_or("");
    if expected.is_empty() || provided != expected {
        return Err(ApiError::new(StatusCode::UNAUTHORIZED, "Invalid or missing admin secret"));
    }
    let refreshed: bool = match state
        .api_keys
        .refresh_debounced(&state.api_key_repo, 5)
        .await
    {
        Ok(value) => value,
        Err(_) => return Err(ApiError::internal("Refresh failed")),
    };
    Ok(ApiOk::new(serde_json::json!({ "refreshed": refreshed })))
}

fn check_admin_secret(headers: &HeaderMap) -> Result<(), ApiError> {
    let expected = std::env::var("INTERNAL_ADMIN_SECRET").unwrap_or_default();
    let provided = headers.get("X-Internal-Secret").and_then(|v| v.to_str().ok()).unwrap_or("");
    if expected.is_empty() || provided != expected {
        return Err(ApiError::new(StatusCode::UNAUTHORIZED, "Invalid or missing admin secret"));
    }
    Ok(())
}

async fn usage(State(state): State<AppState>, headers: HeaderMap, Path(key_hash): Path<String>) -> ApiResult {
    check_admin_secret(&headers)?;
    let record = state.api_keys.get(&key_hash).ok_or_else(|| ApiError::new(StatusCode::NOT_FOUND, "Unknown key"))?;
    let plan = record.effective_plan();
    Ok(ApiOk::new(json!({
        "discord_id": record.discord_id,
        "plan": plan.as_str(),
        "banned": record.banned,
        "used": state.quotas.peek(&key_hash),
        "limit": plan.daily_quota(),
    })))
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(root))
        .route("/help", get(help))
        .route("/admin/refresh-keys", post(refresh_keys))
        .route("/admin/usage/:key_hash", get(usage))
}
