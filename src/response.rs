use axum::{http::StatusCode, response::{IntoResponse, Response}, Json};
use serde::Serialize;
use serde_json::{json, Value};

/// Standard success envelope: { status, data, error: false }
pub struct ApiOk {
    pub status: StatusCode,
    pub data: Value,
}

impl ApiOk {
    pub fn new(data: impl Serialize) -> Self {
        Self { status: StatusCode::OK, data: json!(data) }
    }
}

impl IntoResponse for ApiOk {
    fn into_response(self) -> Response {
        let body = json!({
            "status": self.status.as_u16(),
            "data": self.data,
            "error": false,
        });
        (self.status, Json(body)).into_response()
    }
}

/// Standard error envelope: { status, data: { error, loc, param_type }, error: true }
#[derive(Debug)]
pub struct ApiError {
    pub status: StatusCode,
    pub message: String,
    pub loc: Option<String>,
    pub param_type: Option<String>,
}

impl ApiError {
    pub fn new(status: StatusCode, message: impl Into<String>) -> Self {
        Self { status, message: message.into(), loc: None, param_type: None }
    }

    pub fn with_loc(mut self, loc: impl Into<String>, param_type: impl Into<String>) -> Self {
        self.loc = Some(loc.into());
        self.param_type = Some(param_type.into());
        self
    }

    pub fn not_found() -> Self {
        Self::new(StatusCode::NOT_FOUND, "I was unable to find something related to that")
            .with_loc("query", "query")
    }

    pub fn bad_request(message: impl Into<String>) -> Self {
        Self::new(StatusCode::BAD_REQUEST, message)
    }

    pub fn validation(message: impl Into<String>, loc: impl Into<String>) -> Self {
        Self::new(StatusCode::UNPROCESSABLE_ENTITY, message).with_loc(loc, "query")
    }

    pub fn internal(message: impl Into<String>) -> Self {
        Self::new(StatusCode::INTERNAL_SERVER_ERROR, message)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let body = json!({
            "status": self.status.as_u16(),
            "data": {
                "error": self.message,
                "loc": self.loc,
                "param_type": self.param_type,
            },
            "error": true,
        });
        (self.status, Json(body)).into_response()
    }
}

pub type ApiResult<T = ApiOk> = Result<T, ApiError>;
