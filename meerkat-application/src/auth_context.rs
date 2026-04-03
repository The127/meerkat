use std::collections::HashSet;

use meerkat_domain::models::member::{MemberId, Sub};
use meerkat_domain::models::org_role::OrgRole;
use meerkat_domain::models::organization::OrganizationId;
use meerkat_domain::models::permission::EffectivePermission;

#[derive(Debug, Clone)]
pub struct AuthContext {
    pub sub: Sub,
    pub org_id: OrganizationId,
    pub org_roles: Vec<OrgRole>,
    pub member_id: MemberId,
    pub preferred_name: String,
    pub permissions: HashSet<EffectivePermission>,
}

impl AuthContext {
    pub fn has_permission(&self, permission: impl Into<EffectivePermission>) -> bool {
        self.permissions.contains(&permission.into())
    }
}
