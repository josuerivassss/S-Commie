use axum::{http::header, response::{Html, IntoResponse}, routing::get, Router};
use crate::state::AppState;

// Embedded at compile time: zero runtime file I/O, always ships with the binary.
const OPENAPI_JSON: &str = include_str!("../../static/openapi.json");

async fn openapi_spec() -> impl IntoResponse {
    ([(header::CONTENT_TYPE, "application/json")], OPENAPI_JSON)
}

/// Minimal Swagger UI page, loaded from CDN — no Rust dependency, no build cost.
async fn swagger_ui() -> impl IntoResponse {
    Html(r##"<!DOCTYPE html>
<html>
<head>
  <title>SCOMMIE - Docs</title>
  <link rel="stylesheet" href="https://cdnjs.cloudflare.com/ajax/libs/swagger-ui/5.17.14/swagger-ui.min.css">
</head>
<body>
  <div id="swagger-ui"></div>
  <script src="https://cdnjs.cloudflare.com/ajax/libs/swagger-ui/5.17.14/swagger-ui-bundle.min.js"></script>
  <script>
    window.onload = () => SwaggerUIBundle({ url: "/openapi.json", dom_id: "#swagger-ui" });
  </script>
</body>
</html>"##)
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/openapi.json", get(openapi_spec))
        .route("/docs", get(swagger_ui))
}