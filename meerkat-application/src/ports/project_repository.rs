use async_trait::async_trait;

use meerkat_domain::models::project::{Project, ProjectId, ProjectIdentifier};

use crate::error::ApplicationError;

#[async_trait]
#[cfg_attr(any(test, feature = "test-utils"), mockall::automock)]
pub trait ProjectRepository: Send + Sync {
    fn add(&self, project: Project);
    fn save(&self, project: Project);
    fn delete(&self, id: ProjectId);
    async fn find(&self, identifier: &ProjectIdentifier) -> Result<Project, ApplicationError>;
}
