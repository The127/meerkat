use chrono::{DateTime, Utc};

use meerkat_domain::models::member::MemberId;
use meerkat_domain::models::project::ProjectId;
use meerkat_domain::models::project_role::ProjectRoleId;

use crate::error::ApplicationError;

#[derive(Debug, Clone)]
pub struct ProjectMemberReadModel {
    pub member_id: MemberId,
    pub preferred_name: String,
    pub sub: String,
    pub role_id: ProjectRoleId,
    pub role_name: String,
    pub created_at: DateTime<Utc>,
}

#[async_trait::async_trait]
#[cfg_attr(any(test, feature = "test-utils"), mockall::automock)]
pub trait ProjectMemberReadStore: Send + Sync {
    async fn list_by_project(
        &self,
        project_id: &ProjectId,
    ) -> Result<Vec<ProjectMemberReadModel>, ApplicationError>;
}
