use meerkat_domain::models::permission::ProjectPermission;
use meerkat_domain::models::project::ProjectId;
use meerkat_domain::models::project_role::{ProjectRoleId, ProjectRoleSlug};

use crate::error::ApplicationError;

#[derive(Debug, Clone)]
pub struct ProjectRoleReadModel {
    pub id: ProjectRoleId,
    pub name: String,
    pub slug: ProjectRoleSlug,
    pub permissions: Vec<ProjectPermission>,
    pub is_default: bool,
}

#[async_trait::async_trait]
#[cfg_attr(any(test, feature = "test-utils"), mockall::automock)]
pub trait ProjectRoleReadStore: Send + Sync {
    async fn list_by_project(
        &self,
        project_id: &ProjectId,
    ) -> Result<Vec<ProjectRoleReadModel>, ApplicationError>;
}
