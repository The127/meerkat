use std::str::FromStr;
use std::sync::Arc;

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::{Extension, Json};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

use meerkat_application::context::RequestContext;
use meerkat_application::error::ApplicationError;
use meerkat_application::issues::get::GetIssue;
use meerkat_application::issues::ignore::IgnoreIssue;
use meerkat_application::issues::list::ListIssues;
use meerkat_application::issues::list_events::ListIssueEvents;
use meerkat_application::issues::reopen::ReopenIssue;
use meerkat_application::issues::resolve::ResolveIssue;
use meerkat_application::ports::issue_read_store::IssueReadModel;
use meerkat_application::search::SearchFilter;
use meerkat_domain::models::event::EventId;
use meerkat_domain::models::issue::{IssueNumber, IssueStatus};
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
pub(crate) struct IssueDto {
    #[serde(rename = "id")]
    pub id: String,
    #[serde(rename = "issue_number")]
    pub issue_number: i64,
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

impl From<IssueReadModel> for IssueDto {
    fn from(i: IssueReadModel) -> Self {
        Self {
            id: i.id.as_uuid().to_string(),
            issue_number: i.issue_number,
            title: i.title,
            fingerprint_hash: i.fingerprint_hash,
            status: i.status,
            level: i.level,
            event_count: i.event_count,
            first_seen: i.first_seen,
            last_seen: i.last_seen,
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct ListIssuesResponseDto {
    #[serde(rename = "items")]
    pub items: Vec<IssueDto>,
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
    let status = status_filter.status
        .as_deref()
        .map(IssueStatus::from_str)
        .transpose()
        .map_err(|_| ApplicationError::Validation(
            format!("invalid status filter: '{}'", status_filter.status.as_deref().unwrap_or_default()),
        ))?;

    let query = ListIssues {
        project: ProjectIdentifier::Slug(resolved_org.id, slug),
        search: search.search.as_deref().and_then(SearchFilter::new),
        status: status.map(|s| s.to_string()),
        limit: pagination.limit(),
        offset: pagination.offset(),
    };

    let result = state.mediator.dispatch(query, &req_ctx).await?;

    let items = result.items.into_iter().map(IssueDto::from).collect();

    Ok(Json(ListIssuesResponseDto {
        items,
        total: result.total,
    }))
}

// --- Resolve ---

#[utoipa::path(
    post,
    path = "/api/v1/projects/{slug}/issues/{issue_number}/resolve",
    responses(
        (status = 204, description = "Issue resolved"),
        (status = 404, description = "Issue not found"),
    )
)]
pub(crate) async fn resolve_issue(
    State(state): State<AppState>,
    Extension(req_ctx): Extension<Arc<RequestContext>>,
    Extension(resolved_org): Extension<ResolvedOrganization>,
    Path((slug, issue_number)): Path<(ProjectSlug, u64)>,
) -> Result<StatusCode, ApiError> {
    let cmd = ResolveIssue {
        project: ProjectIdentifier::Slug(resolved_org.id, slug),
        issue_number: IssueNumber::new(issue_number),
    };

    state.mediator.dispatch(cmd, &req_ctx).await?;

    Ok(StatusCode::NO_CONTENT)
}

// --- Reopen ---

#[utoipa::path(
    post,
    path = "/api/v1/projects/{slug}/issues/{issue_number}/reopen",
    responses(
        (status = 204, description = "Issue reopened"),
        (status = 404, description = "Issue not found"),
    )
)]
pub(crate) async fn reopen_issue(
    State(state): State<AppState>,
    Extension(req_ctx): Extension<Arc<RequestContext>>,
    Extension(resolved_org): Extension<ResolvedOrganization>,
    Path((slug, issue_number)): Path<(ProjectSlug, u64)>,
) -> Result<StatusCode, ApiError> {
    let cmd = ReopenIssue {
        project: ProjectIdentifier::Slug(resolved_org.id, slug),
        issue_number: IssueNumber::new(issue_number),
    };

    state.mediator.dispatch(cmd, &req_ctx).await?;

    Ok(StatusCode::NO_CONTENT)
}

// --- Ignore ---

#[utoipa::path(
    post,
    path = "/api/v1/projects/{slug}/issues/{issue_number}/ignore",
    responses(
        (status = 204, description = "Issue ignored"),
        (status = 404, description = "Issue not found"),
    )
)]
pub(crate) async fn ignore_issue(
    State(state): State<AppState>,
    Extension(req_ctx): Extension<Arc<RequestContext>>,
    Extension(resolved_org): Extension<ResolvedOrganization>,
    Path((slug, issue_number)): Path<(ProjectSlug, u64)>,
) -> Result<StatusCode, ApiError> {
    let cmd = IgnoreIssue {
        project: ProjectIdentifier::Slug(resolved_org.id, slug),
        issue_number: IssueNumber::new(issue_number),
    };

    state.mediator.dispatch(cmd, &req_ctx).await?;

    Ok(StatusCode::NO_CONTENT)
}

// --- Get Issue ---

#[utoipa::path(
    get,
    path = "/api/v1/projects/{slug}/issues/{issue_number}",
    responses(
        (status = 200, description = "Issue detail", body = IssueDto),
        (status = 404, description = "Issue not found"),
    )
)]
pub(crate) async fn get_issue(
    State(state): State<AppState>,
    Extension(req_ctx): Extension<Arc<RequestContext>>,
    Extension(resolved_org): Extension<ResolvedOrganization>,
    Path((slug, issue_number)): Path<(ProjectSlug, i64)>,
) -> Result<Json<IssueDto>, ApiError> {
    let query = GetIssue {
        project: ProjectIdentifier::Slug(resolved_org.id, slug),
        issue_number,
    };

    let issue = state.mediator.dispatch(query, &req_ctx).await?;

    Ok(Json(IssueDto::from(issue)))
}

// --- List Issue Events ---

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct EventListItemDto {
    #[serde(rename = "id")]
    pub id: EventId,
    #[serde(rename = "message")]
    pub message: String,
    #[serde(rename = "level")]
    pub level: String,
    #[serde(rename = "platform")]
    pub platform: String,
    #[serde(rename = "timestamp")]
    pub timestamp: DateTime<Utc>,
    #[serde(rename = "server_name")]
    pub server_name: Option<String>,
    #[serde(rename = "environment")]
    pub environment: Option<String>,
    #[serde(rename = "release")]
    pub release: Option<String>,
    #[serde(rename = "exception_type")]
    pub exception_type: Option<String>,
    #[serde(rename = "exception_value")]
    pub exception_value: Option<String>,
    #[serde(rename = "tags")]
    pub tags: Vec<EventTagDto>,
    #[serde(rename = "extra")]
    pub extra: serde_json::Value,
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct EventTagDto {
    #[serde(rename = "key")]
    pub key: String,
    #[serde(rename = "value")]
    pub value: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct ListIssueEventsResponseDto {
    #[serde(rename = "items")]
    pub items: Vec<EventListItemDto>,
    #[serde(rename = "total")]
    pub total: i64,
}

#[utoipa::path(
    get,
    path = "/api/v1/projects/{slug}/issues/{issue_number}/events",
    params(PaginationQueryDto),
    responses(
        (status = 200, description = "List of events for issue", body = ListIssueEventsResponseDto),
        (status = 404, description = "Project not found"),
    )
)]
pub(crate) async fn list_issue_events(
    State(state): State<AppState>,
    Extension(req_ctx): Extension<Arc<RequestContext>>,
    Extension(resolved_org): Extension<ResolvedOrganization>,
    Path((slug, issue_number)): Path<(ProjectSlug, i64)>,
    Query(pagination): Query<PaginationQueryDto>,
) -> Result<Json<ListIssueEventsResponseDto>, ApiError> {
    let query = ListIssueEvents {
        project: ProjectIdentifier::Slug(resolved_org.id, slug),
        issue_number,
        limit: pagination.limit(),
        offset: pagination.offset(),
    };

    let result = state.mediator.dispatch(query, &req_ctx).await?;

    let items = result
        .items
        .into_iter()
        .map(|e| EventListItemDto {
            id: e.id,
            message: e.message,
            level: e.level,
            platform: e.platform,
            timestamp: e.timestamp,
            server_name: e.server_name,
            environment: e.environment,
            release: e.release,
            exception_type: e.exception_type,
            exception_value: e.exception_value,
            tags: e.tags.into_iter().map(|(k, v)| EventTagDto { key: k, value: v }).collect(),
            extra: e.extra,
        })
        .collect();

    Ok(Json(ListIssueEventsResponseDto {
        items,
        total: result.total,
    }))
}
