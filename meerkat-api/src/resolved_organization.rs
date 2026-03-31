use meerkat_domain::models::organization::{OrganizationId, OrganizationSlug};

#[derive(Debug, Clone)]
pub struct ResolvedOrganization {
    pub id: OrganizationId,
    pub slug: OrganizationSlug,
    pub is_master: bool,
}
