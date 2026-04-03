use meerkat_domain::models::oidc_config::{Audience, ClaimMapping, ClientId, OidcConfigId, OidcConfigStatus};
use meerkat_domain::models::organization::OrganizationId;
use meerkat_domain::shared::url::Url;

use crate::error::ApplicationError;

#[derive(Debug, Clone)]
pub struct OidcConfigReadModel {
    pub id: OidcConfigId,
    pub organization_id: OrganizationId,
    pub name: String,
    pub client_id: ClientId,
    pub issuer_url: Url,
    pub audience: Audience,
    pub discovery_url: Option<Url>,
    pub claim_mapping: ClaimMapping,
    pub status: OidcConfigStatus,
}

#[async_trait::async_trait]
#[cfg_attr(any(test, feature = "test-utils"), mockall::automock)]
pub trait OidcConfigReadStore: Send + Sync {
    async fn find_active_by_org_id(
        &self,
        org_id: &OrganizationId,
    ) -> Result<OidcConfigReadModel, ApplicationError>;

    async fn find_by_issuer_and_audience(
        &self,
        issuer_url: &Url,
        audience: &Audience,
    ) -> Result<Option<OidcConfigReadModel>, ApplicationError>;

    async fn list_by_org_id(
        &self,
        org_id: &OrganizationId,
    ) -> Result<Vec<OidcConfigReadModel>, ApplicationError>;
}
