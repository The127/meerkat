use chrono::{DateTime, Utc};

use meerkat_domain::models::issue::IssueId;
use meerkat_domain::models::project::ProjectId;

use crate::error::ApplicationError;
use crate::ports::project_read_store::PagedResult;
use crate::search::SearchFilter;

#[derive(Debug, Clone)]
pub struct IssueReadModel {
    pub id: IssueId,
    pub project_id: ProjectId,
    pub title: String,
    pub fingerprint_hash: String,
    pub status: String,
    pub level: String,
    pub event_count: i64,
    pub first_seen: DateTime<Utc>,
    pub last_seen: DateTime<Utc>,
}

#[async_trait::async_trait]
#[cfg_attr(any(test, feature = "test-utils"), mockall::automock)]
pub trait IssueReadStore: Send + Sync {
    async fn find_by_id(
        &self,
        issue_id: &IssueId,
    ) -> Result<Option<IssueReadModel>, ApplicationError>;

    async fn list_by_project(
        &self,
        project_id: &ProjectId,
        status: Option<&str>,
        search: Option<&SearchFilter>,
        limit: i64,
        offset: i64,
    ) -> Result<PagedResult<IssueReadModel>, ApplicationError>;
}
