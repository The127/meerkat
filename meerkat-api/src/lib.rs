use axum::Router;
use axum::routing::{delete, get, post};
use utoipa::OpenApi;
use crate::handlers::{health, ingest, issues, members, oidc, oidc_admin, organizations, project_keys, projects, team};
use crate::state::AppState;

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
        members::get_current_user,
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
        oidc_admin::list_oidc_configs,
        oidc_admin::add_oidc_config,
        oidc_admin::activate_oidc_config,
        oidc_admin::delete_oidc_config,
        oidc_admin::update_oidc_claim_mapping,
        oidc_admin::list_oidc_config_warnings,
        oidc_admin::dismiss_oidc_config_warning,
        team::list_members,
        team::list_project_roles,
        team::list_project_members,
        team::list_member_projects,
        team::get_member_access,
    ),
    components(schemas(
        error::ErrorDto,
        health::HealthDto,
        members::CurrentUserDto,
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
        oidc_admin::OidcConfigListItemDto,
        oidc_admin::AddOidcConfigRequestDto,
        oidc_admin::AddOidcConfigResponseDto,
        oidc_admin::OidcConfigWarningDto,
        team::MemberDto,
        team::ProjectRoleDto,
        team::ProjectMemberDto,
        team::MemberProjectDto,
        team::MemberAccessDto,
        team::OrgAccessDto,
        team::ProjectAccessDto,
        team::ProjectRoleAccessDto,
    ))
)]
struct ApiDoc;

pub fn router(state: AppState) -> Router {
    let org_routes = Router::new()
        .route("/", post(organizations::create_organization));

    let project_routes = Router::new()
        .route("/", get(projects::list_projects).post(projects::create_project))
        .route("/{slug}", get(projects::get_project).delete(projects::delete_project))
        .route("/{slug}/rename", post(projects::rename_project))
        .route("/{slug}/roles", get(team::list_project_roles))
        .route("/{slug}/members", get(team::list_project_members))
        .route("/{slug}/issues", get(issues::list_issues))
        .route("/{slug}/issues/{issue_number}", get(issues::get_issue))
        .route("/{slug}/issues/{issue_number}/events", get(issues::list_issue_events))
        .route("/{slug}/issues/{issue_number}/resolve", post(issues::resolve_issue))
        .route("/{slug}/issues/{issue_number}/reopen", post(issues::reopen_issue))
        .route("/{slug}/issues/{issue_number}/ignore", post(issues::ignore_issue))
        .route("/{slug}/keys", get(project_keys::list_project_keys).post(project_keys::create_project_key))
        .route("/{slug}/keys/{key_id}", delete(project_keys::revoke_project_key))
        .route("/{slug}/keys/{key_id}/rate-limit", post(project_keys::update_project_key_rate_limit));

    let mut protected_routes = Router::new()
        .nest("/api/v1/organizations", org_routes)
        .nest("/api/v1/projects", project_routes)
        .route("/api/v1/organization/rename", post(organizations::rename_organization))
        .route("/api/v1/organization", delete(organizations::delete_organization))
        .route("/api/v1/organization/oidc-configs", get(oidc_admin::list_oidc_configs).post(oidc_admin::add_oidc_config))
        .route("/api/v1/organization/oidc-configs/{id}/activate", post(oidc_admin::activate_oidc_config))
        // TODO: revisit — PUT on a sub-resource doesn't match our POST-for-mutations convention.
        // Consider folding into a general "update OIDC config" endpoint.
        .route("/api/v1/organization/oidc-configs/{id}/claim-mapping", axum::routing::put(oidc_admin::update_oidc_claim_mapping))
        .route("/api/v1/organization/oidc-configs/{id}", delete(oidc_admin::delete_oidc_config))
        .route("/api/v1/organization/oidc-configs/{id}/warnings", get(oidc_admin::list_oidc_config_warnings))
        .route("/api/v1/organization/oidc-configs/{id}/warnings/{warning_key}", delete(oidc_admin::dismiss_oidc_config_warning))
        .route("/api/v1/me", get(members::get_current_user))
        .route("/api/v1/members", get(team::list_members))
        .route("/api/v1/members/{id}/projects", get(team::list_member_projects))
        .route("/api/v1/members/{id}/access", get(team::get_member_access))
        .layer(axum::middleware::from_fn_with_state(state.clone(), middleware::request_context));

    if state.auth_enabled {
        protected_routes = protected_routes
            .layer(axum::middleware::from_fn_with_state(state.clone(), middleware::authenticate));
    }

    let api_v1_routes = Router::new()
        .merge(protected_routes)
        .route("/api/v1/oidc", get(oidc::get_oidc_config))
        .route("/api/v1/organization", get(organizations::get_organization))
        .layer(axum::middleware::from_fn_with_state(state.clone(), middleware::request_context))
        .layer(axum::middleware::from_fn_with_state(state.clone(), middleware::resolve_subdomain));

    let ingest_routes = Router::new()
        .route("/api/v1/ingest", post(ingest::ingest_event));

    Router::new()
        .merge(api_v1_routes)
        .merge(ingest_routes)
        .route("/api/openapi.json", get(|| async { axum::Json(ApiDoc::openapi()) }))
        .route("/health", get(health::liveness))
        .route("/health/ready", get(health::readiness))
        .layer(axum::middleware::from_fn_with_state(state.clone(), middleware::error_observer))
        .with_state(state)
}
