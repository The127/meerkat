use meerkat_domain::models::organization::OrganizationId;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct AuthContext {
    pub sub: String,
    pub org_id: OrganizationId,
}
