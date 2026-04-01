use chrono::{DateTime, Utc};

use meerkat_domain::models::organization::OrganizationId;
use meerkat_domain::models::project::{ProjectId, ProjectSlug};

use crate::error::ApplicationError;
use crate::search::SearchFilter;

#[derive(Debug, Clone)]
pub struct ProjectReadModel {
    pub id: ProjectId,
    pub organization_id: OrganizationId,
    pub name: String,
    pub slug: ProjectSlug,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct PagedResult<T> {
    pub items: Vec<T>,
    pub total: i64,
}

#[async_trait::async_trait]
#[cfg_attr(any(test, feature = "test-utils"), mockall::automock)]
pub trait ProjectReadStore: Send + Sync {
    async fn list_by_org(
        &self,
        org_id: &OrganizationId,
        search: Option<&SearchFilter>,
        limit: i64,
        offset: i64,
    ) -> Result<PagedResult<ProjectReadModel>, ApplicationError>;
}
