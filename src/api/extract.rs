//! Request extractors that report every failure as a FastAPI-style 422.

use axum::Json;
use axum::extract::{FromRequest, FromRequestParts, Path, Query, Request};
use axum::http::request::Parts;
use serde::de::DeserializeOwned;

use crate::domain::liability::LiabilityFields;
use crate::domain::patch::LiabilityPatch;
use crate::dto::ListParams;
use crate::error::AppError;

/// Request bodies that know how to validate themselves after parsing.
pub trait Validated {
    fn validate_body(&self) -> Result<(), AppError>;
}

impl Validated for LiabilityFields {
    fn validate_body(&self) -> Result<(), AppError> {
        self.check()
    }
}

impl Validated for LiabilityPatch {
    fn validate_body(&self) -> Result<(), AppError> {
        self.check()
    }
}

/// JSON body, parsed and validated; any failure (including bad syntax) is a 422.
pub struct ValidatedJson<T>(pub T);

impl<S, T> FromRequest<S> for ValidatedJson<T>
where
    S: Send + Sync,
    T: DeserializeOwned + Validated,
{
    type Rejection = AppError;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let Json(value) = Json::<T>::from_request(req, state)
            .await
            .map_err(|e| AppError::invalid(&["body"], e.body_text()))?;
        value.validate_body()?;
        Ok(Self(value))
    }
}

/// `{client_id}` path segment; a malformed path is a 422.
pub struct ClientId(pub String);

impl<S: Send + Sync> FromRequestParts<S> for ClientId {
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let Path(id) = Path::<String>::from_request_parts(parts, state)
            .await
            .map_err(|e| AppError::invalid(&["path"], e.body_text()))?;
        Ok(Self(id))
    }
}

/// List-endpoint query string with `skip >= 0` and `1 <= limit <= 200` enforced.
pub struct ListQuery {
    pub params: ListParams,
    pub skip: u64,
    pub limit: i64,
}

impl<S: Send + Sync> FromRequestParts<S> for ListQuery {
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let Query(params) = Query::<ListParams>::from_request_parts(parts, state)
            .await
            .map_err(|e| AppError::invalid(&["query"], e.body_text()))?;
        let limit = params.limit.unwrap_or(50);
        if !(1..=200).contains(&limit) {
            return Err(AppError::invalid(
                &["query", "limit"],
                "must be between 1 and 200",
            ));
        }
        let skip = params.skip.unwrap_or(0);
        Ok(Self {
            params,
            skip,
            limit: limit as i64,
        })
    }
}
