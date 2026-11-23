use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;
use serde_json::json;

/// One validation problem, shaped like FastAPI's 422 entries.
#[derive(Debug, Clone, Serialize)]
pub struct ErrorDetail {
    pub loc: Vec<String>,
    pub msg: String,
    #[serde(rename = "type")]
    pub kind: &'static str,
}

impl ErrorDetail {
    pub fn new(loc: &[&str], msg: impl Into<String>) -> Self {
        Self {
            loc: loc.iter().map(|s| (*s).to_owned()).collect(),
            msg: msg.into(),
            kind: "value_error",
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("{0}")]
    NotFound(&'static str),
    #[error("request validation failed")]
    Validation(Vec<ErrorDetail>),
    #[error("database error: {0}")]
    Db(#[from] mongodb::error::Error),
    #[error("stored document is invalid: {0}")]
    Codec(String),
}

impl AppError {
    pub fn invalid(loc: &[&str], msg: impl Into<String>) -> Self {
        Self::Validation(vec![ErrorDetail::new(loc, msg)])
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        match self {
            Self::NotFound(msg) => {
                (StatusCode::NOT_FOUND, Json(json!({ "detail": msg }))).into_response()
            }
            Self::Validation(details) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(json!({ "detail": details })),
            )
                .into_response(),
            Self::Db(_) | Self::Codec(_) => {
                tracing::error!(error = %self, "request failed");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "detail": "Internal server error" })),
                )
                    .into_response()
            }
        }
    }
}
