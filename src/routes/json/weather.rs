use crate::extract::Query;
use crate::{response::{ApiError, ApiOk, ApiResult}, state::AppState, validate};
use axum::extract::State;
use axum::http::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Deserialize)]
pub struct WeatherQuery {
    query: String,
}

#[derive(Serialize)]
struct WeatherOut {
    description: String,
    feels_like_f: i32,
    humidity: i32,
    precipitation_in: f32,
    pressure: i32,
    temperature_f: i32,
    visibility_miles: i32,
    wind_direction: String,
    wind_speed_mph: i32,
}

/// GET /json/weather?query=City — hits wttr.in's `j1` JSON endpoint
pub async fn handler(State(state): State<AppState>, Query(q): Query<WeatherQuery>) -> ApiResult {
    validate::len(&q.query, 1, 150, "query")?;

    if !state.external_breakers.weather.allow() {
        return Err(ApiError::new(StatusCode::SERVICE_UNAVAILABLE, "Weather service temporarily unavailable, try again shortly"));
    }

    let location = q.query.replace(' ', "+");
    let res = state
        .http
        .get(format!("https://wttr.in/{location}"))
        .query(&[("format", "j1")])
        .send()
        .await;

    let res = match res {
        Ok(r) if r.status().is_success() => r,
        _ => {
            state.external_breakers.weather.record_failure();
            return Err(ApiError::not_found());
        }
    };

    let body: Value = match res.json().await {
        Ok(b) => b,
        Err(_) => {
            state.external_breakers.weather.record_failure();
            return Err(ApiError::not_found());
        }
    };

    let current = match body.get("current_condition").and_then(|c| c.get(0)) {
        Some(c) => c,
        None => {
            state.external_breakers.weather.record_failure();
            return Err(ApiError::not_found());
        }
    };

    let str_field = |key: &str| current.get(key).and_then(Value::as_str).unwrap_or("").to_string();
    let int_field = |key: &str| str_field(key).parse::<i32>().unwrap_or(0);
    let float_field = |key: &str| str_field(key).parse::<f32>().unwrap_or(0.0);

    let description = current
        .get("weatherDesc")
        .and_then(|d| d.get(0))
        .and_then(|d| d.get("value"))
        .and_then(Value::as_str)
        .unwrap_or("")
        .trim()
        .to_string();

    state.external_breakers.weather.record_success();

    Ok(ApiOk::new(WeatherOut {
        description,
        feels_like_f: int_field("FeelsLikeF"),
        humidity: int_field("humidity"),
        precipitation_in: float_field("precipInches"),
        pressure: int_field("pressure"),
        temperature_f: int_field("temp_F"),
        visibility_miles: int_field("visibilityMiles"),
        wind_direction: str_field("winddir16Point"),
        wind_speed_mph: int_field("windspeedMiles"),
    }))
}