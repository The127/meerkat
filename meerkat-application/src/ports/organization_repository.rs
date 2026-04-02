use async_trait::async_trait;

use meerkat_domain::models::organization::{Organization, OrganizationId, OrganizationIdentifier};

use crate::error::ApplicationError;

#[async_trait]
#[cfg_attr(any(test, feature = "test-utils"), mockall::automock)]
pub trait OrganizationRepository: Send + Sync {
    fn add(&self, org: Organization);
    fn save(&self, org: Organization);
    fn delete(&self, id: OrganizationId);
    async fn find(&self, identifier: &OrganizationIdentifier) -> Result<Organization, ApplicationError>;
}
