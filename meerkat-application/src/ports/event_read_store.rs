use chrono::{DateTime, Utc};

use meerkat_domain::models::event::EventId;
use meerkat_domain::models::issue::IssueId;
use meerkat_domain::models::project::ProjectId;

use crate::error::ApplicationError;
use crate::ports::project_read_store::PagedResult;

#[derive(Debug, Clone)]
pub struct EventReadModel {
    pub id: EventId,
    pub project_id: ProjectId,
    pub issue_id: IssueId,
    pub fingerprint_hash: String,
    pub message: String,
    pub level: String,
    pub platform: String,
    pub timestamp: DateTime<Utc>,
    pub server_name: Option<String>,
    pub environment: Option<String>,
    pub release: Option<String>,
    pub exception_type: Option<String>,
    pub exception_value: Option<String>,
    pub tags: Vec<(String, String)>,
    pub extra: serde_json::Value,
}

#[async_trait::async_trait]
#[cfg_attr(any(test, feature = "test-utils"), mockall::automock)]
pub trait EventReadStore: Send + Sync {
    async fn list_by_issue(
        &self,
        issue_id: &IssueId,
        limit: i64,
        offset: i64,
    ) -> Result<PagedResult<EventReadModel>, ApplicationError>;
}
