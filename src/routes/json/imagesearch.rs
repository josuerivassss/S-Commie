use crate::extract::Query;
use crate::{response::{ApiError, ApiOk, ApiResult}, state::AppState, validate};
use axum::extract::State;
use axum::http::StatusCode;
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

/// Distinguishes "backend reachable but returned nothing" from an actual
/// network/parsing failure -- only the latter should trip the breaker.
enum BackendOutcome {
    Found(Vec<String>),
    EmptyButReachable,
    Failed,
}

/// GET /json/imagesearch?query=&limit= — tries DuckDuckGo images first, falls to Bing.
pub async fn handler(State(state): State<AppState>, Query(q): Query<ImageSearchQuery>) -> ApiResult {
    validate::len(&q.query, 1, 150, "query")?;
    let limit = q.limit.clamp(1, 30);

    if !state.external_breakers.imagesearch.allow() {
        return Err(ApiError::new(StatusCode::SERVICE_UNAVAILABLE, "Image search temporarily unavailable, try again shortly"));
    }

    let mut any_failed = false;

    match duckduckgo_images(&state.http, &q.query, limit).await {
        BackendOutcome::Found(results) => {
            state.external_breakers.imagesearch.record_success();
            return Ok(ApiOk::new(ImageSearchOut { query: q.query, backend: "duckduckgo".into(), results }));
        }
        BackendOutcome::EmptyButReachable => {}
        BackendOutcome::Failed => any_failed = true,
    }

    match bing_images(&state.http, &q.query, limit).await {
        BackendOutcome::Found(results) => {
            state.external_breakers.imagesearch.record_success();
            return Ok(ApiOk::new(ImageSearchOut { query: q.query, backend: "bing".into(), results }));
        }
        BackendOutcome::EmptyButReachable => {}
        BackendOutcome::Failed => any_failed = true,
    }

    // Only trips the breaker if at least one backend actually errored --
    // two reachable-but-empty responses is a legitimate "no results", not
    // a service outage.
    if any_failed {
        state.external_breakers.imagesearch.record_failure();
    }
    Err(ApiError::not_found())
}

async fn duckduckgo_images(http: &reqwest::Client, query: &str, limit: usize) -> BackendOutcome {
    let Ok(vqd_re) = Regex::new(r#"vqd=['"]?([\d-]+)"#) else { return BackendOutcome::Failed };

    let Ok(page_res) = http.get("https://duckduckgo.com/").query(&[("q", query)]).header("User-Agent", "Mozilla/5.0").send().await else {
        return BackendOutcome::Failed;
    };
    let Ok(page) = page_res.text().await else { return BackendOutcome::Failed };
    let Some(vqd) = vqd_re.captures(&page).and_then(|c| c.get(1)).map(|m| m.as_str().to_string()) else {
        return BackendOutcome::Failed;
    };

    let Ok(res) = http
        .get("https://duckduckgo.com/i.js")
        .query(&[("l", "us-en"), ("o", "json"), ("q", query), ("vqd", vqd.as_str()), ("f", ",,,"), ("p", "1")])
        .header("User-Agent", "Mozilla/5.0")
        .header("Referer", "https://duckduckgo.com/")
        .send()
        .await
    else {
        return BackendOutcome::Failed;
    };

    if !res.status().is_success() {
        return BackendOutcome::Failed; // e.g. 202/403 — DDG is rate-limiting, let Bing handle it
    }

    let Ok(body) = res.json::<Value>().await else { return BackendOutcome::Failed };
    let Some(results) = body.get("results").and_then(Value::as_array) else { return BackendOutcome::Failed };

    let images: Vec<String> = results.iter().filter_map(|r| r.get("image").and_then(Value::as_str).map(String::from)).take(limit).collect();
    if images.is_empty() {
        BackendOutcome::EmptyButReachable
    } else {
        BackendOutcome::Found(images)
    }
}

async fn bing_images(http: &reqwest::Client, query: &str, limit: usize) -> BackendOutcome {
    let Ok(murl_re) = Regex::new(r#"murl&quot;:&quot;(.*?)&quot;"#) else { return BackendOutcome::Failed };

    let Ok(res) = http.get("https://www.bing.com/images/async").query(&[("q", query), ("adlt", "on")]).send().await else {
        return BackendOutcome::Failed;
    };
    if !res.status().is_success() {
        return BackendOutcome::Failed;
    }
    let Ok(body) = res.text().await else { return BackendOutcome::Failed };

    let results: Vec<String> = murl_re.captures_iter(&body).filter_map(|c| c.get(1).map(|m| m.as_str().to_string())).take(limit).collect();

    if results.is_empty() {
        BackendOutcome::EmptyButReachable
    } else {
        BackendOutcome::Found(results)
    }
}