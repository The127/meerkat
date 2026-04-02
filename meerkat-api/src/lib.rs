use axum::Router;
use axum::routing::{delete, get, post};
use utoipa::OpenApi;
use crate::handlers::{health, oidc, organizations, projects};
use crate::state::AppState;

pub(crate) mod auth_context;
pub mod error;
pub mod handlers;
mod middleware;
pub(crate) mod pagination;
pub mod resolved_organization;
pub(crate) mod search;
pub mod state;

#[derive(OpenApi)]
#[openapi(
    paths(
        health::liveness,
        oidc::get_oidc_config,
        organizations::create_organization,
        organizations::get_organization,
        organizations::rename_organization,
        organizations::delete_organization,
        projects::create_project,
        projects::list_projects,
        projects::get_project,
        projects::rename_project,
        projects::delete_project,
    ),
    components(schemas(
        error::ErrorDto,
        health::HealthDto,
        oidc::OidcConfigDto,
        organizations::CreateOrganizationRequestDto,
        organizations::CreateOrganizationResponseDto,
        organizations::OrganizationDto,
        organizations::RenameOrganizationRequestDto,
        projects::CreateProjectRequestDto,
        projects::CreateProjectResponseDto,
        projects::ListProjectsResponseDto,
        projects::ProjectDto,
        projects::ProjectListItemDto,
        projects::RenameProjectRequestDto,
    ))
)]
struct ApiDoc;

pub fn router(state: AppState) -> Router {
    let org_routes = Router::new()
        .route("/", post(organizations::create_organization));

    let project_routes = Router::new()
        .route("/", get(projects::list_projects).post(projects::create_project))
        .route("/{slug}", get(projects::get_project).delete(projects::delete_project))
        .route("/{slug}/rename", post(projects::rename_project));

    let mut protected_routes = Router::new()
        .nest("/api/v1/organizations", org_routes)
        .nest("/api/v1/projects", project_routes)
        .route("/api/v1/organization/rename", post(organizations::rename_organization))
        .route("/api/v1/organization", delete(organizations::delete_organization));

    if state.auth_enabled {
        protected_routes = protected_routes
            .layer(axum::middleware::from_fn_with_state(state.clone(), middleware::authenticate));
    }

    let api_v1_routes = Router::new()
        .merge(protected_routes)
        .route("/api/v1/oidc", get(oidc::get_oidc_config))
        .route("/api/v1/organization", get(organizations::get_organization))
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
