use async_trait::async_trait;

use meerkat_domain::models::member::MemberId;
use meerkat_domain::models::project::ProjectId;
use meerkat_domain::models::project_member::ProjectMember;

use crate::error::ApplicationError;

#[async_trait]
#[cfg_attr(any(test, feature = "test-utils"), mockall::automock)]
pub trait ProjectMemberRepository: Send + Sync {
    fn add(&self, member: ProjectMember);
    fn save(&self, member: ProjectMember);
    async fn find_by_project_and_member(
        &self,
        project_id: &ProjectId,
        member_id: &MemberId,
    ) -> Result<Option<ProjectMember>, ApplicationError>;
}

#[cfg(any(test, feature = "test-utils"))]
pub struct NoOpProjectMemberRepository;

#[cfg(any(test, feature = "test-utils"))]
#[async_trait]
impl ProjectMemberRepository for NoOpProjectMemberRepository {
    fn add(&self, _member: ProjectMember) {}
    fn save(&self, _member: ProjectMember) {}
    async fn find_by_project_and_member(
        &self,
        _project_id: &ProjectId,
        _member_id: &MemberId,
    ) -> Result<Option<ProjectMember>, ApplicationError> {
        Ok(None)
    }
}
