use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::get;
use axum::{Json, Router};

use super::AppState;
use super::caller::CallerContext;
use super::extract::{ClientId, ListQuery, ValidatedJson};
use crate::domain::enums::CallerType;
use crate::domain::liability::LiabilityFields;
use crate::domain::patch::LiabilityPatch;
use crate::dto::{LiabilityList, LiabilitySummary, LiabilityView, ListParams};
use crate::error::AppError;
use crate::service::to_view;

const NOT_FOUND: AppError = AppError::NotFound("Liability not found");

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_liabilities).post(create_liability))
        // Static segment wins over "/{liability_id}", so "summary" is never treated as an ID.
        .route("/summary", get(liabilities_summary))
        .route(
            "/{liability_id}",
            get(get_liability)
                .patch(update_liability)
                .delete(delete_liability),
        )
}

#[utoipa::path(
    post,
    path = "/api/v1/clients/{client_id}/liabilities",
    tag = "liabilities",
    summary = "Create a liability",
    request_body = LiabilityFields,
    params(
        ("client_id" = String, Path),
        ("X-Caller-Type" = CallerType, Header, description = "advisor or client app"),
        ("X-User-Id" = String, Header, description = "Calling user's ID"),
    ),
    responses(
        (status = 201, body = LiabilityView),
        (status = 422, description = "Validation error"),
    )
)]
pub async fn create_liability(
    State(state): State<AppState>,
    ClientId(client_id): ClientId,
    caller: CallerContext,
    ValidatedJson(body): ValidatedJson<LiabilityFields>,
) -> Result<(StatusCode, Json<LiabilityView>), AppError> {
    let doc = state.store.create(&client_id, body, &caller).await?;
    Ok((StatusCode::CREATED, Json(to_view(&doc, &caller))))
}

#[utoipa::path(
    get,
    path = "/api/v1/clients/{client_id}/liabilities",
    tag = "liabilities",
    summary = "List liabilities",
    params(
        ("client_id" = String, Path),
        ListParams,
        ("X-Caller-Type" = CallerType, Header, description = "advisor or client app"),
        ("X-User-Id" = String, Header, description = "Calling user's ID"),
    ),
    responses(
        (status = 200, body = LiabilityList),
        (status = 422, description = "Validation error"),
    )
)]
pub async fn list_liabilities(
    State(state): State<AppState>,
    ClientId(client_id): ClientId,
    caller: CallerContext,
    query: ListQuery,
) -> Result<Json<LiabilityList>, AppError> {
    let list = state
        .store
        .list_for_client(
            &client_id,
            query.params.liability_type,
            query.skip,
            query.limit,
            &caller,
        )
        .await?;
    Ok(Json(list))
}

#[utoipa::path(
    get,
    path = "/api/v1/clients/{client_id}/liabilities/summary",
    tag = "liabilities",
    summary = "Totals by type and annualized payments",
    params(
        ("client_id" = String, Path),
        ("X-Caller-Type" = CallerType, Header, description = "advisor or client app"),
        ("X-User-Id" = String, Header, description = "Calling user's ID"),
    ),
    responses(
        (status = 200, body = LiabilitySummary),
        (status = 422, description = "Validation error"),
    )
)]
pub async fn liabilities_summary(
    State(state): State<AppState>,
    ClientId(client_id): ClientId,
    caller: CallerContext,
) -> Result<Json<LiabilitySummary>, AppError> {
    Ok(Json(state.store.summary(&client_id, &caller).await?))
}

#[utoipa::path(
    get,
    path = "/api/v1/clients/{client_id}/liabilities/{liability_id}",
    tag = "liabilities",
    summary = "Read one liability",
    params(
        ("client_id" = String, Path),
        ("liability_id" = String, Path),
        ("X-Caller-Type" = CallerType, Header, description = "advisor or client app"),
        ("X-User-Id" = String, Header, description = "Calling user's ID"),
    ),
    responses(
        (status = 200, body = LiabilityView),
        (status = 404, description = "Liability not found"),
        (status = 422, description = "Validation error"),
    )
)]
pub async fn get_liability(
    State(state): State<AppState>,
    Path((client_id, liability_id)): Path<(String, String)>,
    caller: CallerContext,
) -> Result<Json<LiabilityView>, AppError> {
    let doc = state
        .store
        .get(&client_id, &liability_id, &caller)
        .await?
        .ok_or(NOT_FOUND)?;
    Ok(Json(to_view(&doc, &caller)))
}

#[utoipa::path(
    patch,
    path = "/api/v1/clients/{client_id}/liabilities/{liability_id}",
    tag = "liabilities",
    summary = "Partial update (staged when the caller is a client)",
    request_body = LiabilityPatch,
    params(
        ("client_id" = String, Path),
        ("liability_id" = String, Path),
        ("X-Caller-Type" = CallerType, Header, description = "advisor or client app"),
        ("X-User-Id" = String, Header, description = "Calling user's ID"),
    ),
    responses(
        (status = 200, body = LiabilityView),
        (status = 404, description = "Liability not found"),
        (status = 422, description = "Validation error"),
    )
)]
pub async fn update_liability(
    State(state): State<AppState>,
    Path((client_id, liability_id)): Path<(String, String)>,
    caller: CallerContext,
    ValidatedJson(body): ValidatedJson<LiabilityPatch>,
) -> Result<Json<LiabilityView>, AppError> {
    let doc = state
        .store
        .update(&client_id, &liability_id, body, &caller)
        .await?
        .ok_or(NOT_FOUND)?;
    Ok(Json(to_view(&doc, &caller)))
}

#[utoipa::path(
    delete,
    path = "/api/v1/clients/{client_id}/liabilities/{liability_id}",
    tag = "liabilities",
    summary = "Hard delete for an advisor; staged for a client",
    params(
        ("client_id" = String, Path),
        ("liability_id" = String, Path),
        ("X-Caller-Type" = CallerType, Header, description = "advisor or client app"),
        ("X-User-Id" = String, Header, description = "Calling user's ID"),
    ),
    responses(
        (status = 204, description = "Deleted, or staged for a client"),
        (status = 404, description = "Liability not found"),
        (status = 422, description = "Validation error"),
    )
)]
pub async fn delete_liability(
    State(state): State<AppState>,
    Path((client_id, liability_id)): Path<(String, String)>,
    caller: CallerContext,
) -> Result<StatusCode, AppError> {
    if state
        .store
        .delete(&client_id, &liability_id, &caller)
        .await?
    {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(NOT_FOUND)
    }
}
