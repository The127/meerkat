use chrono::{DateTime, Utc};

use meerkat_domain::models::member::MemberId;
use meerkat_domain::models::organization::OrganizationId;
use meerkat_domain::models::permission::ProjectPermission;
use meerkat_domain::models::project::{ProjectId, ProjectSlug};
use meerkat_domain::models::project_role::ProjectRoleId;

use crate::error::ApplicationError;

#[derive(Debug, Clone)]
pub struct ProjectMemberRoleReadModel {
    pub role_id: ProjectRoleId,
    pub role_name: String,
}

#[derive(Debug, Clone)]
pub struct ProjectMemberReadModel {
    pub member_id: MemberId,
    pub preferred_name: String,
    pub sub: String,
    pub roles: Vec<ProjectMemberRoleReadModel>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct MemberProjectReadModel {
    pub project_name: String,
    pub project_slug: ProjectSlug,
    pub role_id: ProjectRoleId,
    pub role_name: String,
}

#[derive(Debug, Clone)]
pub struct MemberProjectAccessReadModel {
    pub project_name: String,
    pub project_slug: ProjectSlug,
    pub role_name: String,
    pub role_permissions: Vec<ProjectPermission>,
}

#[async_trait::async_trait]
#[cfg_attr(any(test, feature = "test-utils"), mockall::automock)]
pub trait ProjectMemberReadStore: Send + Sync {
    async fn list_by_project(
        &self,
        project_id: &ProjectId,
    ) -> Result<Vec<ProjectMemberReadModel>, ApplicationError>;

    async fn list_by_member(
        &self,
        member_id: &MemberId,
    ) -> Result<Vec<MemberProjectReadModel>, ApplicationError>;

    async fn list_access_by_member(
        &self,
        member_id: &MemberId,
        org_id: &OrganizationId,
    ) -> Result<Vec<MemberProjectAccessReadModel>, ApplicationError>;
}
