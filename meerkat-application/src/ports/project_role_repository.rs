use async_trait::async_trait;

use meerkat_domain::models::project_role::{ProjectRole, ProjectRoleId};

use crate::error::ApplicationError;

#[async_trait]
#[cfg_attr(any(test, feature = "test-utils"), mockall::automock)]
pub trait ProjectRoleRepository: Send + Sync {
    fn add(&self, role: ProjectRole);
    fn save(&self, role: ProjectRole);
    fn delete(&self, id: ProjectRoleId);
    async fn find(&self, id: &ProjectRoleId) -> Result<ProjectRole, ApplicationError>;
}

#[cfg(any(test, feature = "test-utils"))]
pub struct NoOpProjectRoleRepository;

#[cfg(any(test, feature = "test-utils"))]
#[async_trait]
impl ProjectRoleRepository for NoOpProjectRoleRepository {
    fn add(&self, _role: ProjectRole) {}
    fn save(&self, _role: ProjectRole) {}
    fn delete(&self, _id: ProjectRoleId) {}
    async fn find(&self, _id: &ProjectRoleId) -> Result<ProjectRole, ApplicationError> {
        Err(ApplicationError::NotFound)
    }
}
