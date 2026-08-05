use crate::extract::Query;
use crate::{response::{ApiError, ApiOk, ApiResult}, state::AppState, validate};
use axum::extract::State;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Deserialize)]
pub struct TranslateQuery {
    text: String,
    #[serde(default = "default_source")]
    source: String,
    target: String,
}
fn default_source() -> String {
    "auto".to_string()
}

#[derive(Serialize)]
struct TranslateOut {
    source: String,
    target: String,
    original: String,
    translated: String,
    backend: String,
}

/// GET /json/translate?text=&source=auto&target=en
pub async fn handler(State(state): State<AppState>, Query(q): Query<TranslateQuery>) -> ApiResult {
    validate::len(&q.text, 1, 1500, "text")?;
    validate::len(&q.target, 2, 8, "target")?;

    if let Some((translated, detected_source)) = google_translate(&state, &q.text, &q.source, &q.target).await {
        return Ok(ApiOk::new(TranslateOut {
            source: detected_source,
            target: q.target,
            original: q.text,
            translated,
            backend: "google".into(),
        }));
    }

    if q.source != "auto" {
        if let Some(translated) = mymemory_translate(&state, &q.text, &q.source, &q.target).await {
            return Ok(ApiOk::new(TranslateOut {
                source: q.source.clone(),
                target: q.target,
                original: q.text,
                translated,
                backend: "mymemory".into(),
            }));
        }
    }

    Err(ApiError::bad_request("The provided translation is invalid"))
}

async fn google_translate(state: &AppState, text: &str, source: &str, target: &str) -> Option<(String, String)> {
    state.translate_throttle.wait().await;

    let res = state
        .http
        .get("https://translate.googleapis.com/translate_a/single")
        .query(&[("client", "gtx"), ("sl", source), ("tl", target), ("dt", "t"), ("q", text)])
        .send()
        .await
        .ok()?;

    if !res.status().is_success() {
        return None; // e.g. 429 — Google is rate-limiting us, let MyMemory handle it
    }

    let body: Value = res.json().await.ok()?;

    // Response shape: [[["translated chunk", "original chunk", null, null, 3], ...], null, "detected_source_lang"]
    let translated = body.get(0)?.as_array()?.iter().filter_map(|s| s.get(0).and_then(Value::as_str)).collect::<String>();
    if translated.is_empty() {
        return None;
    }

    let detected_source = body.get(2).and_then(Value::as_str).unwrap_or(source).to_string();
    Some((translated, detected_source))
}

async fn mymemory_translate(state: &AppState, text: &str, source: &str, target: &str) -> Option<String> {
    let langpair = format!("{source}|{target}");
    let res = state
        .http
        .get("https://api.mymemory.translated.net/get")
        .query(&[("q", text), ("langpair", langpair.as_str())])
        .send()
        .await
        .ok()?;

    if !res.status().is_success() {
        return None;
    }

    let body: Value = res.json().await.ok()?;
    if body.get("responseStatus").and_then(Value::as_i64) != Some(200) {
        return None;
    }
    body.get("responseData")?.get("translatedText")?.as_str().map(String::from)
}