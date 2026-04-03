use chrono::{DateTime, Utc};

use meerkat_domain::models::project::ProjectId;
use meerkat_domain::models::project_key::{ProjectKeyId, ProjectKeyStatus};

use crate::error::ApplicationError;
use crate::ports::project_read_store::PagedResult;
use crate::search::SearchFilter;

#[derive(Debug, Clone)]
pub struct ProjectKeyReadModel {
    pub id: ProjectKeyId,
    pub project_id: ProjectId,
    pub key_token: String,
    pub label: String,
    pub status: ProjectKeyStatus,
    pub created_at: DateTime<Utc>,
}

#[async_trait::async_trait]
#[cfg_attr(any(test, feature = "test-utils"), mockall::automock)]
pub trait ProjectKeyReadStore: Send + Sync {
    async fn list_by_project(
        &self,
        project_id: &ProjectId,
        search: Option<&SearchFilter>,
        limit: i64,
        offset: i64,
    ) -> Result<PagedResult<ProjectKeyReadModel>, ApplicationError>;
}
