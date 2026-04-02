use meerkat_domain::models::member::{MemberId, Sub};
use meerkat_domain::models::org_role::OrgRole;
use meerkat_domain::models::organization::OrganizationId;

#[derive(Debug, Clone)]
pub struct AuthContext {
    pub sub: Sub,
    pub org_id: OrganizationId,
    pub org_roles: Vec<OrgRole>,
    pub member_id: MemberId,
}
