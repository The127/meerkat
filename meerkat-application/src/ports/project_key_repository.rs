use async_trait::async_trait;

use meerkat_domain::models::project_key::{ProjectKey, ProjectKeyId};

use crate::error::ApplicationError;

#[async_trait]
#[cfg_attr(any(test, feature = "test-utils"), mockall::automock)]
pub trait ProjectKeyRepository: Send + Sync {
    fn add(&self, key: ProjectKey);
    fn save(&self, key: ProjectKey);
    async fn find(&self, id: &ProjectKeyId) -> Result<ProjectKey, ApplicationError>;
}

#[cfg(any(test, feature = "test-utils"))]
pub struct NoOpProjectKeyRepository;

#[cfg(any(test, feature = "test-utils"))]
#[async_trait]
impl ProjectKeyRepository for NoOpProjectKeyRepository {
    fn add(&self, _key: ProjectKey) {}
    fn save(&self, _key: ProjectKey) {}
    async fn find(&self, _id: &ProjectKeyId) -> Result<ProjectKey, ApplicationError> {
        Err(ApplicationError::NotFound)
    }
}
