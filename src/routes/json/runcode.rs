use crate::extract::{Json, Query};
use crate::{codewrap, response::{ApiError, ApiOk, ApiResult}, state::AppState, validate};
use axum::{
    extract::{ConnectInfo, State},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{net::SocketAddr, time::Duration};

#[derive(Deserialize)]
pub struct RunCodeQuery {
    /// When true, bare snippets get auto-wrapped into a runnable `main` for
    /// languages that need one (java, c/c++, c#, go, rust, scala).
    #[serde(default, rename = "tryParsingMain")]
    try_parsing_main: bool,
}

#[derive(Deserialize)]
pub struct RunCodeBody {
    language: String,
    code: String,
    #[serde(default)]
    args: Vec<String>,
    #[serde(default)]
    stdin: String,
}

#[derive(Serialize)]
struct RunCodeOut {
    language: String,
    version: String,
    output: String,
    stdout: String,
    stderr: String,
    compile_stderr: Option<String>,
    truncated: bool,
}

const MAX_OUTPUT_CHARS: usize = 8000;
const MAX_ARGS: usize = 20;
const MAX_ARG_LEN: usize = 200;
const MAX_STDIN: usize = 2000;

/// POST /json/runcode?tryParsingMain=false
pub async fn handler(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Query(q): Query<RunCodeQuery>,
    Json(body): Json<RunCodeBody>,
) -> ApiResult {
    validate::len(&body.language, 1, 30, "language")?;
    validate::len(&body.code, 1, 3000, "code")?;
    if body.args.len() > MAX_ARGS {
        return Err(ApiError::validation(format!("at most {MAX_ARGS} args allowed"), "args"));
    }
    for arg in &body.args {
        validate::len(arg, 0, MAX_ARG_LEN, "args")?;
    }
    validate::len(&body.stdin, 0, MAX_STDIN, "stdin")?;

    if !state.runcode_limiter.check(addr.ip()) {
        return Err(ApiError::new(StatusCode::TOO_MANY_REQUESTS, "Too many code executions from this address, slow down"));
    }

    if !state.languages.is_loaded() {
        return Err(ApiError::internal("Language list not loaded yet, try again shortly"));
    }
    let (language, version) = state.languages.resolve(&body.language).ok_or_else(|| {
        let mut available = state.languages.all_aliases();
        available.truncate(50);
        ApiError::validation(format!("Language does not exist: {}. Some supported values: {}", body.language, available.join(", ")), "language")
    })?;

    let source = if q.try_parsing_main { codewrap::wrap_if_needed(&language, &body.code) } else { body.code };

    // Bound concurrent executions in flight to Piston, regardless of caller.
    let _permit = state.runcode_semaphore.acquire().await.map_err(|_| ApiError::internal("Server busy, try again shortly"))?;

    let payload = serde_json::json!({
        "language": language,
        "version": version,
        "files": [{ "content": source }],
        "args": body.args,
        "stdin": body.stdin,
        "log": 0,
    });

    let res = state
        .http
        .post("https://emkc.org/api/v2/piston/execute")
        .json(&payload)
        .timeout(Duration::from_secs(20)) // compiling can be slow; longer than the client's default 10s
        .send()
        .await
        .map_err(|_| ApiError::internal("Execution service unavailable"))?;

    if !res.status().is_success() {
        return Err(ApiError::internal("Execution service returned an error"));
    }

    let result: Value = res.json().await.map_err(|_| ApiError::internal("Invalid response from execution service"))?;
    let run = result.get("run").ok_or_else(|| ApiError::internal("Unexpected response from execution service"))?;

    let compile_stderr = result.get("compile").and_then(|c| c.get("stderr")).and_then(Value::as_str).map(str::to_string);
    let stdout = run.get("stdout").and_then(Value::as_str).unwrap_or("").to_string();
    let stderr = run.get("stderr").and_then(Value::as_str).unwrap_or("").to_string();
    let mut output = run.get("output").and_then(Value::as_str).unwrap_or("").to_string();

    let truncated = output.chars().count() > MAX_OUTPUT_CHARS;
    if truncated {
        output = output.chars().take(MAX_OUTPUT_CHARS).collect::<String>() + "[...]";
    }

    Ok(ApiOk::new(RunCodeOut { language, version, output, stdout, stderr, compile_stderr, truncated }))
}