use crate::extract::Query;
use crate::{response::{ApiError, ApiOk, ApiResult}, state::AppState, validate};
use axum::extract::State;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Deserialize)]
pub struct ImageSearchQuery {
    query: String,
    #[serde(default = "default_limit")]
    limit: usize,
}
fn default_limit() -> usize {
    10
}

#[derive(Serialize)]
struct ImageSearchOut {
    query: String,
    backend: String,
    results: Vec<String>,
}

/// GET /json/imagesearch?query=&limit= — tries DuckDuckGo images first, falls to Bing.
pub async fn handler(State(state): State<AppState>, Query(q): Query<ImageSearchQuery>) -> ApiResult {
    validate::len(&q.query, 1, 150, "query")?;
    let limit = q.limit.clamp(1, 30);

    if let Some(results) = duckduckgo_images(&state.http, &q.query, limit).await {
        if !results.is_empty() {
            return Ok(ApiOk::new(ImageSearchOut { query: q.query, backend: "duckduckgo".into(), results }));
        }
    }

    if let Some(results) = bing_images(&state.http, &q.query, limit).await {
        if !results.is_empty() {
            return Ok(ApiOk::new(ImageSearchOut { query: q.query, backend: "bing".into(), results }));
        }
    }

    Err(ApiError::not_found())
}

async fn duckduckgo_images(http: &reqwest::Client, query: &str, limit: usize) -> Option<Vec<String>> {
    let vqd_re = Regex::new(r#"vqd=['"]?([\d-]+)"#).ok()?;

    let page = http
        .get("https://duckduckgo.com/")
        .query(&[("q", query)])
        .header("User-Agent", "Mozilla/5.0")
        .send()
        .await
        .ok()?
        .text()
        .await
        .ok()?;
    let vqd = vqd_re.captures(&page)?.get(1)?.as_str().to_string();

    let res = http
        .get("https://duckduckgo.com/i.js")
        .query(&[("l", "us-en"), ("o", "json"), ("q", query), ("vqd", vqd.as_str()), ("f", ",,,"), ("p", "1")])
        .header("User-Agent", "Mozilla/5.0")
        .header("Referer", "https://duckduckgo.com/")
        .send()
        .await
        .ok()?;

    if !res.status().is_success() {
        return None; // e.g. 202/403 — DDG is rate-limiting, let the Bing fallback handle it
    }

    let body: Value = res.json().await.ok()?;
    let results = body.get("results")?.as_array()?;
    Some(results.iter().filter_map(|r| r.get("image").and_then(Value::as_str).map(String::from)).take(limit).collect())
}

/// async image-results HTML fragment.
async fn bing_images(http: &reqwest::Client, query: &str, limit: usize) -> Option<Vec<String>> {
    let murl_re = Regex::new(r#"murl&quot;:&quot;(.*?)&quot;"#).ok()?;
    let body = http.get("https://www.bing.com/images/async").query(&[("q", query), ("adlt", "on")]).send().await.ok()?.text().await.ok()?;

    let results: Vec<String> = murl_re.captures_iter(&body).filter_map(|c| c.get(1).map(|m| m.as_str().to_string())).take(limit).collect();

    if results.is_empty() {
        None
    } else {
        Some(results)
    }
}