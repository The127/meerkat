use chrono::{DateTime, Utc};

use meerkat_domain::models::member::MemberId;
use meerkat_domain::models::org_role::OrgRole;
use meerkat_domain::models::organization::OrganizationId;
use meerkat_domain::models::project::ProjectSlug;

use crate::error::ApplicationError;
use crate::ports::project_read_store::PagedResult;
use crate::search::SearchFilter;

#[derive(Debug, Clone)]
pub struct MemberReadModel {
    pub id: MemberId,
    pub sub: String,
    pub preferred_name: String,
    pub org_roles: Vec<OrgRole>,
    pub created_at: DateTime<Utc>,
    pub last_seen: DateTime<Utc>,
}

pub struct ListMembersQuery {
    pub org_id: OrganizationId,
    pub search: Option<SearchFilter>,
    pub role: Option<OrgRole>,
    pub project_slug: Option<ProjectSlug>,
    pub limit: i64,
    pub offset: i64,
}

#[async_trait::async_trait]
#[cfg_attr(any(test, feature = "test-utils"), mockall::automock)]
pub trait MemberReadStore: Send + Sync {
    async fn list_by_org(
        &self,
        query: &ListMembersQuery,
    ) -> Result<PagedResult<MemberReadModel>, ApplicationError>;

    async fn find_member_for_access(
        &self,
        member_id: &MemberId,
        org_id: &OrganizationId,
    ) -> Result<Option<MemberReadModel>, ApplicationError>;
}
