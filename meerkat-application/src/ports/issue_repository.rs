use meerkat_domain::models::issue::Issue;
use meerkat_domain::models::project::ProjectId;

use crate::error::ApplicationError;

#[async_trait::async_trait]
#[cfg_attr(any(test, feature = "test-utils"), mockall::automock)]
pub trait IssueRepository: Send + Sync {
    async fn find_by_fingerprint(
        &self,
        project_id: &ProjectId,
        fingerprint_hash: &str,
    ) -> Result<Option<Issue>, ApplicationError>;

    async fn add(&self, issue: &Issue) -> Result<(), ApplicationError>;

    async fn save(&self, issue: &Issue) -> Result<(), ApplicationError>;
}
