use axum::Router;
use axum::routing::{get, post};
use utoipa::OpenApi;
use crate::handlers::{health, organizations, projects};
use crate::state::AppState;

pub mod error;
pub mod handlers;
mod middleware;
pub mod resolved_organization;
pub mod state;

#[derive(OpenApi)]
#[openapi(
    paths(
        health::liveness,
        organizations::create_organization,
        projects::create_project,
    ),
    components(schemas(
        health::HealthDto,
        organizations::CreateOrganizationRequestDto,
        organizations::CreateOrganizationResponseDto,
        projects::CreateProjectRequestDto,
        projects::CreateProjectResponseDto,
    ))
)]
struct ApiDoc;

pub fn router(state: AppState) -> Router {
    let org_routes = Router::new()
        .route("/", post(organizations::create_organization));

    let project_routes = Router::new()
        .route("/", post(projects::create_project));

    let api_v1_routes = Router::new()
        .nest("/api/v1/organizations", org_routes)
        .nest("/api/v1/projects", project_routes)
        .layer(axum::middleware::from_fn_with_state(state.clone(), middleware::resolve_subdomain));

    Router::new()
        .merge(api_v1_routes)
        .route("/api/openapi.json", get(|| async { axum::Json(ApiDoc::openapi()) }))
        .route("/health", get(health::liveness))
        .route("/health/ready", get(health::readiness))
        .layer(axum::middleware::from_fn_with_state(state.clone(), middleware::error_observer))
        .layer(axum::middleware::from_fn_with_state(state.clone(), middleware::request_context))
        .with_state(state)
}
