use std::sync::Arc;

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::{Extension, Json};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use meerkat_application::context::RequestContext;
use meerkat_application::projects::create::CreateProject;
use meerkat_application::projects::delete::DeleteProject;
use meerkat_application::projects::get::GetProject;
use meerkat_application::projects::list::ListProjects;
use meerkat_application::projects::rename::RenameProject;
use meerkat_application::search::SearchFilter;
use meerkat_domain::models::organization::OrganizationId;
use meerkat_domain::models::project::{ProjectId, ProjectIdentifier, ProjectSlug};


use crate::error::ApiError;
use crate::pagination::PaginationQueryDto;
use crate::resolved_organization::ResolvedOrganization;
use crate::search::SearchQueryDto;
use crate::state::AppState;

#[derive(Debug, Deserialize, ToSchema)]
pub(crate) struct RenameProjectRequestDto {
    #[serde(rename = "name")]
    pub name: String,
}

#[utoipa::path(
    post,
    path = "/api/v1/projects/{slug}/rename",
    request_body = RenameProjectRequestDto,
    responses(
        (status = 204, description = "Project renamed"),
        (status = 400, description = "Validation error"),
        (status = 404, description = "Project not found"),
        (status = 409, description = "Conflict"),
    )
)]
pub(crate) async fn rename_project(
    State(state): State<AppState>,
    Extension(req_ctx): Extension<Arc<RequestContext>>,
    Extension(resolved_org): Extension<ResolvedOrganization>,
    Path(slug): Path<ProjectSlug>,
    Json(body): Json<RenameProjectRequestDto>,
) -> Result<StatusCode, ApiError> {
    let cmd = RenameProject {
        identifier: ProjectIdentifier::Slug(resolved_org.id, slug),
        name: body.name,
    };

    state.mediator.dispatch(cmd, &req_ctx).await?;

    Ok(StatusCode::NO_CONTENT)
}

#[derive(Debug, Deserialize, ToSchema)]
pub(crate) struct CreateProjectRequestDto {
    #[serde(rename = "organization_id")]
    pub organization_id: OrganizationId,
    #[serde(rename = "name")]
    pub name: String,
    #[serde(rename = "slug")]
    pub slug: ProjectSlug,
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct CreateProjectResponseDto {
    #[serde(rename = "id")]
    pub id: ProjectId,
}

#[utoipa::path(
    post,
    path = "/api/v1/projects",
    request_body = CreateProjectRequestDto,
    responses(
        (status = 201, description = "Project created", body = CreateProjectResponseDto),
        (status = 400, description = "Validation error"),
        (status = 500, description = "Internal server error"),
    )
)]
pub(crate) async fn create_project(
    State(state): State<AppState>,
    Extension(req_ctx): Extension<Arc<RequestContext>>,
    Json(body): Json<CreateProjectRequestDto>,
) -> Result<(StatusCode, Json<CreateProjectResponseDto>), ApiError> {
    let cmd = CreateProject {
        organization_id: body.organization_id,
        name: body.name,
        slug: body.slug,
    };

    let id = state.mediator.dispatch(cmd, &req_ctx).await?;

    Ok((StatusCode::CREATED, Json(CreateProjectResponseDto { id })))
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct ProjectListItemDto {
    #[serde(rename = "id")]
    pub id: ProjectId,
    #[serde(rename = "organization_id")]
    pub organization_id: OrganizationId,
    #[serde(rename = "name")]
    pub name: String,
    #[serde(rename = "slug")]
    pub slug: ProjectSlug,
    #[serde(rename = "created_at")]
    pub created_at: DateTime<Utc>,
    #[serde(rename = "updated_at")]
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct ListProjectsResponseDto {
    #[serde(rename = "items")]
    pub items: Vec<ProjectListItemDto>,
    #[serde(rename = "total")]
    pub total: i64,
}

#[utoipa::path(
    get,
    path = "/api/v1/projects",
    params(PaginationQueryDto, SearchQueryDto),
    responses(
        (status = 200, description = "List of projects", body = ListProjectsResponseDto),
        (status = 500, description = "Internal server error"),
    )
)]
pub(crate) async fn list_projects(
    State(state): State<AppState>,
    Extension(req_ctx): Extension<Arc<RequestContext>>,
    Extension(resolved_org): Extension<ResolvedOrganization>,
    Query(pagination): Query<PaginationQueryDto>,
    Query(search): Query<SearchQueryDto>,
) -> Result<Json<ListProjectsResponseDto>, ApiError> {
    let query = ListProjects {
        org_id: resolved_org.id,
        search: search.search.as_deref().and_then(SearchFilter::new),
        limit: pagination.limit(),
        offset: pagination.offset(),
    };

    let result = state.mediator.dispatch(query, &req_ctx).await?;

    let items = result
        .items
        .into_iter()
        .map(|p| ProjectListItemDto {
            id: p.id,
            organization_id: p.organization_id,
            name: p.name,
            slug: p.slug,
            created_at: p.created_at,
            updated_at: p.updated_at,
        })
        .collect();

    Ok(Json(ListProjectsResponseDto {
        items,
        total: result.total,
    }))
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct ProjectDto {
    #[serde(rename = "id")]
    pub id: ProjectId,
    #[serde(rename = "organization_id")]
    pub organization_id: OrganizationId,
    #[serde(rename = "name")]
    pub name: String,
    #[serde(rename = "slug")]
    pub slug: ProjectSlug,
    #[serde(rename = "created_at")]
    pub created_at: DateTime<Utc>,
    #[serde(rename = "updated_at")]
    pub updated_at: DateTime<Utc>,
}

#[utoipa::path(
    get,
    path = "/api/v1/projects/{slug}",
    responses(
        (status = 200, description = "Project found", body = ProjectDto),
        (status = 404, description = "Project not found"),
        (status = 500, description = "Internal server error"),
    )
)]
pub(crate) async fn get_project(
    State(state): State<AppState>,
    Extension(req_ctx): Extension<Arc<RequestContext>>,
    Extension(resolved_org): Extension<ResolvedOrganization>,
    Path(slug): Path<ProjectSlug>,
) -> Result<Json<ProjectDto>, ApiError> {
    let query = GetProject {
        project: ProjectIdentifier::Slug(resolved_org.id, slug),
    };

    let p = state.mediator.dispatch(query, &req_ctx).await?;

    Ok(Json(ProjectDto {
        id: p.id,
        organization_id: p.organization_id,
        name: p.name,
        slug: p.slug,
        created_at: p.created_at,
        updated_at: p.updated_at,
    }))
}

#[utoipa::path(
    delete,
    path = "/api/v1/projects/{slug}",
    responses(
        (status = 204, description = "Project deleted"),
        (status = 404, description = "Project not found"),
    )
)]
pub(crate) async fn delete_project(
    State(state): State<AppState>,
    Extension(req_ctx): Extension<Arc<RequestContext>>,
    Extension(resolved_org): Extension<ResolvedOrganization>,
    Path(slug): Path<ProjectSlug>,
) -> Result<StatusCode, ApiError> {
    let cmd = DeleteProject {
        identifier: ProjectIdentifier::Slug(resolved_org.id, slug),
    };

    state.mediator.dispatch(cmd, &req_ctx).await?;

    Ok(StatusCode::NO_CONTENT)
}
