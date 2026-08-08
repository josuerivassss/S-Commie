use crate::apikeys::model::KeyRecord;
use crate::response::ApiError;
use crate::state::AppState;
use axum::{extract::{Request, State}, http::{HeaderMap, StatusCode}, middleware::Next, response::Response};
use sha2::{Digest, Sha256};

const TOKEN_HEADER: &str = "TOKEN";

fn hash_key(raw: &str) -> String {
    format!("{:x}", Sha256::digest(raw.as_bytes())) as String
}

/// Auth + quota gate for /json/*. Resolves everything from in-memory
/// state -- zero Mongo access per request.
pub async fn api_key_auth(State(state): State<AppState>, headers: HeaderMap, request: Request, next: Next) -> Result<Response, ApiError> {
    let token = headers
        .get(TOKEN_HEADER)
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| ApiError::new(StatusCode::UNAUTHORIZED, "Missing TOKEN header"))?;

    let key_hash = hash_key(token);
    let record: KeyRecord = state.api_keys.get(&key_hash).ok_or_else(|| ApiError::new(StatusCode::UNAUTHORIZED, "Invalid API key"))?;

    if record.banned {
        return Err(ApiError::new(StatusCode::FORBIDDEN, "This API key has been banned"));
    }

    let plan = record.effective_plan();
    if let Some(limit) = plan.daily_quota() {
        if !state.quotas.check(&key_hash, limit) {
            return Err(ApiError::new(StatusCode::TOO_MANY_REQUESTS, "Daily quota exceeded for this key"));
        }
    }
    // Partner (unlimited) never touches the quota DashMap.

    Ok(next.run(request).await)
}