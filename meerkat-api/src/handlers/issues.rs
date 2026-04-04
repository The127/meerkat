use std::sync::Arc;

use axum::extract::{Path, Query, State};
use axum::{Extension, Json};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

use meerkat_application::context::RequestContext;
use meerkat_application::issues::list::ListIssues;
use meerkat_application::search::SearchFilter;
use meerkat_domain::models::issue::IssueId;
use meerkat_domain::models::project::{ProjectIdentifier, ProjectSlug};

use crate::error::ApiError;
use crate::pagination::PaginationQueryDto;
use crate::resolved_organization::ResolvedOrganization;
use crate::search::SearchQueryDto;
use crate::state::AppState;

// --- List ---

#[derive(Debug, Deserialize, IntoParams, ToSchema)]
pub(crate) struct IssueStatusFilterQueryDto {
    #[serde(rename = "status")]
    pub status: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct IssueListItemDto {
    #[serde(rename = "id")]
    pub id: IssueId,
    #[serde(rename = "title")]
    pub title: String,
    #[serde(rename = "fingerprint_hash")]
    pub fingerprint_hash: String,
    #[serde(rename = "status")]
    pub status: String,
    #[serde(rename = "level")]
    pub level: String,
    #[serde(rename = "event_count")]
    pub event_count: i64,
    #[serde(rename = "first_seen")]
    pub first_seen: DateTime<Utc>,
    #[serde(rename = "last_seen")]
    pub last_seen: DateTime<Utc>,
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct ListIssuesResponseDto {
    #[serde(rename = "items")]
    pub items: Vec<IssueListItemDto>,
    #[serde(rename = "total")]
    pub total: i64,
}

#[utoipa::path(
    get,
    path = "/api/v1/projects/{slug}/issues",
    params(PaginationQueryDto, SearchQueryDto, IssueStatusFilterQueryDto),
    responses(
        (status = 200, description = "List of issues", body = ListIssuesResponseDto),
        (status = 404, description = "Project not found"),
    )
)]
pub(crate) async fn list_issues(
    State(state): State<AppState>,
    Extension(req_ctx): Extension<Arc<RequestContext>>,
    Extension(resolved_org): Extension<ResolvedOrganization>,
    Path(slug): Path<ProjectSlug>,
    Query(pagination): Query<PaginationQueryDto>,
    Query(search): Query<SearchQueryDto>,
    Query(status_filter): Query<IssueStatusFilterQueryDto>,
) -> Result<Json<ListIssuesResponseDto>, ApiError> {
    let query = ListIssues {
        project: ProjectIdentifier::Slug(resolved_org.id, slug),
        search: search.search.as_deref().and_then(SearchFilter::new),
        status: status_filter.status,
        limit: pagination.limit(),
        offset: pagination.offset(),
    };

    let result = state.mediator.dispatch(query, &req_ctx).await?;

    let items = result
        .items
        .into_iter()
        .map(|i| IssueListItemDto {
            id: i.id,
            title: i.title,
            fingerprint_hash: i.fingerprint_hash,
            status: i.status,
            level: i.level,
            event_count: i.event_count,
            first_seen: i.first_seen,
            last_seen: i.last_seen,
        })
        .collect();

    Ok(Json(ListIssuesResponseDto {
        items,
        total: result.total,
    }))
}
