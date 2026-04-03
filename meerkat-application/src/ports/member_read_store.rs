use chrono::{DateTime, Utc};

use meerkat_domain::models::member::MemberId;
use meerkat_domain::models::org_role::OrgRole;
use meerkat_domain::models::organization::OrganizationId;

use crate::error::ApplicationError;

#[derive(Debug, Clone)]
pub struct MemberReadModel {
    pub id: MemberId,
    pub sub: String,
    pub preferred_name: String,
    pub org_roles: Vec<OrgRole>,
    pub created_at: DateTime<Utc>,
}

#[async_trait::async_trait]
#[cfg_attr(any(test, feature = "test-utils"), mockall::automock)]
pub trait MemberReadStore: Send + Sync {
    async fn list_by_org(
        &self,
        org_id: &OrganizationId,
    ) -> Result<Vec<MemberReadModel>, ApplicationError>;
}
