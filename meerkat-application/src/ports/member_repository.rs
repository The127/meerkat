use meerkat_domain::models::member::{MemberId, Sub};
use meerkat_domain::models::org_role::OrgRole;
use meerkat_domain::models::organization::OrganizationId;

use crate::error::ApplicationError;

#[async_trait::async_trait]
#[cfg_attr(any(test, feature = "test-utils"), mockall::automock)]
pub trait MemberRepository: Send + Sync {
    async fn find_or_create(
        &self,
        org_id: &OrganizationId,
        sub: &Sub,
        preferred_name: &str,
        org_roles: &[OrgRole],
    ) -> Result<MemberId, ApplicationError>;
}
