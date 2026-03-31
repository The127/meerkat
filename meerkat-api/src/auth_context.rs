use meerkat_domain::models::organization::OrganizationId;

#[derive(Debug, Clone)]
pub struct AuthContext {
    pub sub: String,
    pub org_id: OrganizationId,
}
