use std::sync::Arc;

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::{Extension, Json};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use meerkat_application::context::RequestContext;
use meerkat_application::project_keys::create::CreateProjectKey;
use meerkat_application::project_keys::list::ListProjectKeys;
use meerkat_application::project_keys::revoke::RevokeProjectKey;
use meerkat_application::search::SearchFilter;
use meerkat_domain::models::project::ProjectSlug;
use meerkat_domain::models::project_key::ProjectKeyId;

use crate::error::ApiError;
use crate::pagination::PaginationQueryDto;
use crate::resolved_organization::ResolvedOrganization;
use crate::search::SearchQueryDto;
use crate::state::AppState;

// --- List ---

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct ProjectKeyListItemDto {
    #[serde(rename = "id")]
    pub id: ProjectKeyId,
    #[serde(rename = "key_token")]
    pub key_token: String,
    #[serde(rename = "label")]
    pub label: String,
    #[serde(rename = "status")]
    pub status: String,
    #[serde(rename = "dsn")]
    pub dsn: String,
    #[serde(rename = "created_at")]
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct ListProjectKeysResponseDto {
    #[serde(rename = "items")]
    pub items: Vec<ProjectKeyListItemDto>,
    #[serde(rename = "total")]
    pub total: i64,
}

#[utoipa::path(
    get,
    path = "/api/v1/projects/{slug}/keys",
    params(PaginationQueryDto, SearchQueryDto),
    responses(
        (status = 200, description = "List of project keys", body = ListProjectKeysResponseDto),
        (status = 404, description = "Project not found"),
    )
)]
pub(crate) async fn list_project_keys(
    State(state): State<AppState>,
    Extension(req_ctx): Extension<Arc<RequestContext>>,
    Extension(resolved_org): Extension<ResolvedOrganization>,
    Path(slug): Path<ProjectSlug>,
    Query(pagination): Query<PaginationQueryDto>,
    Query(search): Query<SearchQueryDto>,
) -> Result<Json<ListProjectKeysResponseDto>, ApiError> {
    let query = ListProjectKeys {
        org_id: resolved_org.id,
        slug: slug.clone(),
        search: search.search.as_deref().and_then(SearchFilter::new),
        limit: pagination.limit(),
        offset: pagination.offset(),
    };

    let result = state.mediator.dispatch(query, &req_ctx).await?;

    let items = result
        .items
        .into_iter()
        .map(|k| {
            let dsn = format!("https://{}@{}/{}", k.key_token, state.base_domain, slug.as_str());
            ProjectKeyListItemDto {
                id: k.id,
                key_token: k.key_token,
                label: k.label,
                status: k.status.to_string(),
                dsn,
                created_at: k.created_at,
            }
        })
        .collect();

    Ok(Json(ListProjectKeysResponseDto {
        items,
        total: result.total,
    }))
}

// --- Create ---

#[derive(Debug, Deserialize, ToSchema)]
pub(crate) struct CreateProjectKeyRequestDto {
    #[serde(rename = "label")]
    pub label: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct CreateProjectKeyResponseDto {
    #[serde(rename = "id")]
    pub id: ProjectKeyId,
}

#[utoipa::path(
    post,
    path = "/api/v1/projects/{slug}/keys",
    request_body = CreateProjectKeyRequestDto,
    responses(
        (status = 201, description = "Project key created", body = CreateProjectKeyResponseDto),
        (status = 400, description = "Validation error"),
        (status = 404, description = "Project not found"),
    )
)]
pub(crate) async fn create_project_key(
    State(state): State<AppState>,
    Extension(req_ctx): Extension<Arc<RequestContext>>,
    Extension(resolved_org): Extension<ResolvedOrganization>,
    Path(slug): Path<ProjectSlug>,
    Json(body): Json<CreateProjectKeyRequestDto>,
) -> Result<(StatusCode, Json<CreateProjectKeyResponseDto>), ApiError> {
    let cmd = CreateProjectKey {
        org_id: resolved_org.id,
        project_slug: slug,
        label: body.label,
    };

    let id = state.mediator.dispatch(cmd, &req_ctx).await?;

    Ok((StatusCode::CREATED, Json(CreateProjectKeyResponseDto { id })))
}

// --- Revoke ---

#[utoipa::path(
    delete,
    path = "/api/v1/projects/{slug}/keys/{key_id}",
    responses(
        (status = 204, description = "Project key revoked"),
        (status = 404, description = "Project key not found"),
    )
)]
pub(crate) async fn revoke_project_key(
    State(state): State<AppState>,
    Extension(req_ctx): Extension<Arc<RequestContext>>,
    Extension(resolved_org): Extension<ResolvedOrganization>,
    Path((slug, key_id)): Path<(ProjectSlug, ProjectKeyId)>,
) -> Result<StatusCode, ApiError> {
    let cmd = RevokeProjectKey {
        org_id: resolved_org.id,
        project_slug: slug,
        key_id,
    };

    state.mediator.dispatch(cmd, &req_ctx).await?;

    Ok(StatusCode::NO_CONTENT)
}
