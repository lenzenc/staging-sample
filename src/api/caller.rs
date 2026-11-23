use axum::extract::FromRequestParts;
use axum::http::HeaderMap;
use axum::http::request::Parts;

use crate::domain::enums::CallerType;
use crate::domain::liability::AuditStamp;
use crate::error::{AppError, ErrorDetail};

pub const CALLER_TYPE_HEADER: &str = "x-caller-type";
pub const USER_ID_HEADER: &str = "x-user-id";

/// Who is calling, from the `X-Caller-Type` and `X-User-Id` headers (not authenticated).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CallerContext {
    pub caller_type: CallerType,
    pub user_id: String,
}

impl CallerContext {
    pub fn stamp(&self) -> AuditStamp {
        AuditStamp::now(self.user_id.clone(), self.caller_type)
    }

    fn from_headers(headers: &HeaderMap) -> Result<Self, AppError> {
        let header = |name: &str| headers.get(name).and_then(|v| v.to_str().ok());
        let mut errors = Vec::new();

        let caller_type = match header(CALLER_TYPE_HEADER) {
            Some("advisor") => Some(CallerType::Advisor),
            Some("client") => Some(CallerType::Client),
            Some(_) => {
                errors.push(ErrorDetail::new(
                    &["header", CALLER_TYPE_HEADER],
                    "must be 'advisor' or 'client'",
                ));
                None
            }
            None => {
                errors.push(ErrorDetail::new(
                    &["header", CALLER_TYPE_HEADER],
                    "field required",
                ));
                None
            }
        };
        let user_id = match header(USER_ID_HEADER) {
            Some(v) if !v.is_empty() => Some(v.to_owned()),
            Some(_) => {
                errors.push(ErrorDetail::new(
                    &["header", USER_ID_HEADER],
                    "must not be empty",
                ));
                None
            }
            None => {
                errors.push(ErrorDetail::new(
                    &["header", USER_ID_HEADER],
                    "field required",
                ));
                None
            }
        };
        match (caller_type, user_id) {
            (Some(caller_type), Some(user_id)) => Ok(Self {
                caller_type,
                user_id,
            }),
            _ => Err(AppError::Validation(errors)),
        }
    }
}

impl<S: Send + Sync> FromRequestParts<S> for CallerContext {
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _: &S) -> Result<Self, Self::Rejection> {
        Self::from_headers(&parts.headers)
    }
}
