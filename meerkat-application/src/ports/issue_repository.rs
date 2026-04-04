use async_trait::async_trait;

use meerkat_domain::models::issue::{Issue, IssueIdentifier};

use crate::error::ApplicationError;

#[async_trait]
#[cfg_attr(any(test, feature = "test-utils"), mockall::automock)]
pub trait IssueRepository: Send + Sync {
    async fn find(&self, identifier: &IssueIdentifier) -> Result<Issue, ApplicationError>;

    fn add(&self, issue: Issue);

    fn save(&self, issue: Issue);
}

#[cfg(any(test, feature = "test-utils"))]
pub struct NoOpIssueRepository;

#[cfg(any(test, feature = "test-utils"))]
#[async_trait]
impl IssueRepository for NoOpIssueRepository {
    async fn find(&self, _identifier: &IssueIdentifier) -> Result<Issue, ApplicationError> {
        Err(ApplicationError::NotFound)
    }

    fn add(&self, _issue: Issue) {}
    fn save(&self, _issue: Issue) {}
}
