use std::sync::Arc;

use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::{Extension, Json};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

use meerkat_application::context::RequestContext;
use meerkat_application::projects::create::CreateProject;
use meerkat_domain::models::organization::OrganizationId;
use meerkat_domain::models::project::{ProjectId, ProjectSlug};

use crate::error::ApiError;
use crate::resolved_organization::ResolvedOrganization;
use crate::state::AppState;

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

#[derive(Debug, Deserialize, IntoParams)]
pub(crate) struct ListProjectsQueryDto {
    #[serde(flatten)]
    pub pagination: crate::pagination::PaginationQueryDto,
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
    params(ListProjectsQueryDto),
    responses(
        (status = 200, description = "List of projects", body = ListProjectsResponseDto),
        (status = 500, description = "Internal server error"),
    )
)]
pub(crate) async fn list_projects(
    State(state): State<AppState>,
    Extension(resolved_org): Extension<ResolvedOrganization>,
    Query(query): Query<ListProjectsQueryDto>,
) -> Result<Json<ListProjectsResponseDto>, ApiError> {
    let result = state
        .project_read_store
        .list_by_org(&resolved_org.id, query.pagination.limit(), query.pagination.offset())
        .await?;

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
