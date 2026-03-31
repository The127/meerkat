use meerkat_domain::models::organization::{OrganizationId, OrganizationSlug};

use crate::error::ApplicationError;

#[derive(Debug, Clone)]
pub struct OrganizationReadModel {
    pub id: OrganizationId,
    pub slug: OrganizationSlug,
    pub name: String,
}

#[async_trait::async_trait]
#[cfg_attr(any(test, feature = "test-utils"), mockall::automock)]
pub trait OrganizationReadStore: Send + Sync {
    async fn any_exists(&self) -> Result<bool, ApplicationError>;

    async fn find_by_slug(
        &self,
        slug: &OrganizationSlug,
    ) -> Result<Option<OrganizationReadModel>, ApplicationError>;
}
