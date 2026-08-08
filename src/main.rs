mod extract;
mod imaging;
mod managers;
mod response;
mod routes;
mod state;
mod throttle;
mod validate;
mod apikeys;
mod circuit;

use axum::{
    extract::{ConnectInfo, State},
    http::Method,
    middleware::{self, Next},
    response::IntoResponse,
    Router,
};
use response::ApiError;
use state::AppState;
use std::{net::SocketAddr, time::Instant, time::Duration};
use tower_http::{cors::CorsLayer, limit::RequestBodyLimitLayer, trace::TraceLayer};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()))
        .init();

    let state = AppState::init().await?; // antes: AppState::init()?

    if let Err(e) = state.api_keys.refresh(&state.api_key_repo).await {
        tracing::warn!(%e, "initial api_keys load failed, starting with an empty cache");
    }
    {
        let state = state.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(120));
            loop {
                interval.tick().await;
                if let Err(e) = state.api_keys.refresh(&state.api_key_repo).await {
                    tracing::warn!(%e, "api_keys refresh failed, keeping previous cache");
                }
            }
        });
    }
    {
        let state = state.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(600));
            loop {
                interval.tick().await;
                state.quotas.cleanup();
            }
        });
    }

    let cors = CorsLayer::new()
        .allow_origin(tower_http::cors::Any)
        .allow_methods([Method::GET, Method::POST])
        .allow_headers(tower_http::cors::Any);

    let app = Router::new()
        .merge(routes::router(state.clone()))
        .fallback(not_found)
        .layer(middleware::from_fn_with_state(state.clone(), request_logger))
        .layer(TraceLayer::new_for_http())
        .layer(RequestBodyLimitLayer::new(2 * 1024 * 1024)) // 2MB, this API takes no file uploads
        .layer(cors)
        .with_state(state);

    let port: u16 = std::env::var("PORT").ok().and_then(|p| p.parse().ok()).unwrap_or(3000);
    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    tracing::info!("Service Commie API listening on {addr}");
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>())
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

/// Any path that doesn't match a route falls through here, keeping the same error envelope.
async fn not_found() -> impl IntoResponse {
    ApiError::not_found()
}

/// Optional request logger: prints method/path/status/latency, and forwards a
/// Discord embed to WEBHOOK when that env var is set (mirrors the original Python middleware).
async fn request_logger(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    req: axum::extract::Request,
    next: Next,
) -> impl IntoResponse {
    let method = req.method().clone();
    let path = req.uri().to_string();
    let start = Instant::now();

    let response = next.run(req).await;

    let status = response.status();
    let elapsed = start.elapsed().as_secs_f64();
    tracing::info!(%method, %path, %status, elapsed_s = elapsed, client = %addr.ip(), "request");

    if let Ok(webhook) = std::env::var("WEBHOOK") {
        if !webhook.is_empty() {
            let http = state.http.clone();
            let embed = serde_json::json!({
                "embeds": [{
                    "title": "API Request",
                    "description": format!("```{method} {path}```"),
                    "color": 3_447_003,
                    "fields": [
                        {"name": "Method", "value": method.to_string(), "inline": true},
                        {"name": "Time", "value": format!("{elapsed:.3}s"), "inline": true},
                        {"name": "Status", "value": status.as_u16().to_string(), "inline": true},
                    ],
                    "footer": {"text": addr.ip().to_string()},
                }]
            });
            // fire-and-forget, never blocks or fails the actual response
            tokio::spawn(async move {
                let _ = http.post(webhook).json(&embed).send().await;
            });
        }
    }

    response
}

async fn shutdown_signal() {
    tokio::signal::ctrl_c().await.ok();
    tracing::info!("shutting down");
}