pub mod caller;
pub mod extract;
pub mod liabilities;

use std::path::PathBuf;

use axum::extract::State;
use axum::http::{HeaderValue, header};
use axum::routing::get;
use axum::{Json, Router};
use serde_json::{Value, json};
use tower_http::services::ServeDir;
use tower_http::set_header::SetResponseHeaderLayer;
use tower_http::trace::TraceLayer;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::db::Store;
use crate::dto::ScenarioInfo;
use crate::error::AppError;
use crate::seed;

#[derive(Clone)]
pub struct AppState {
    pub store: Store,
}

#[derive(OpenApi)]
#[openapi(
    info(title = "Client Liabilities API", version = "0.1.0"),
    paths(
        liabilities::create_liability,
        liabilities::list_liabilities,
        liabilities::liabilities_summary,
        liabilities::get_liability,
        liabilities::update_liability,
        liabilities::delete_liability,
        health,
        list_scenarios,
    )
)]
struct ApiDoc;

#[utoipa::path(get, path = "/health", tag = "health", responses((status = 200, description = "Service and database are up")))]
async fn health(State(state): State<AppState>) -> Result<Json<Value>, AppError> {
    state.store.ping().await?;
    Ok(Json(json!({ "status": "ok" })))
}

/// Static metadata about the seeded sample clients, used by the browser test harness.
#[utoipa::path(get, path = "/api/v1/seed-scenarios", tag = "seed", responses((status = 200, body = Vec<ScenarioInfo>)))]
async fn list_scenarios() -> Json<Vec<ScenarioInfo>> {
    Json(seed::scenarios().iter().map(seed::Scenario::info).collect())
}

/// The whole application: API under `/api/v1`, `/health`, and the browser harness.
pub fn app(store: Store) -> Router {
    let harness_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("harness");
    // Always revalidate, so harness edits show up without a hard refresh.
    let harness = tower::ServiceBuilder::new()
        .layer(SetResponseHeaderLayer::overriding(
            header::CACHE_CONTROL,
            HeaderValue::from_static("no-cache"),
        ))
        .service(ServeDir::new(harness_dir).append_index_html_on_directories(true));

    Router::new()
        .route("/health", get(health))
        .nest(
            "/api/v1",
            Router::new()
                .nest("/clients/{client_id}/liabilities", liabilities::router())
                .route("/seed-scenarios", get(list_scenarios)),
        )
        .nest_service("/harness", harness)
        .merge(SwaggerUi::new("/docs").url("/openapi.json", ApiDoc::openapi()))
        .layer(TraceLayer::new_for_http())
        .with_state(AppState { store })
}
