use async_trait::async_trait;

use meerkat_domain::models::oidc_config::{Audience, ClaimMapping, ClientId, OidcConfig, OidcConfigId};
use meerkat_domain::models::organization::OrganizationIdentifier;
use meerkat_domain::models::permission::OrgPermission;
use meerkat_domain::shared::url::Url;

use crate::behaviors::authorization::{RequestName, RequiredPermissions};
use crate::context::RequestContext;
use crate::error::ApplicationError;
use crate::extensions::Extensions;
use crate::mediator::{Request, Handler};

pub struct AddOidcConfig {
    pub identifier: OrganizationIdentifier,
    pub name: String,
    pub client_id: ClientId,
    pub issuer_url: Url,
    pub audience: Audience,
    pub discovery_url: Option<Url>,
    pub claim_mapping: ClaimMapping,
}

impl Request for AddOidcConfig {
    type Output = OidcConfigId;

    fn extensions(&self) -> Extensions {
        let mut ext = Extensions::new();
        ext.insert(RequestName("AddOidcConfig".to_string()));
        ext.insert(RequiredPermissions(vec![OrgPermission::OrgManageOidc.into()]));
        ext
    }
}

pub struct AddOidcConfigHandler;

#[async_trait]
impl Handler<AddOidcConfig, ApplicationError, RequestContext> for AddOidcConfigHandler {
    async fn handle(
        &self,
        cmd: AddOidcConfig,
        ctx: &RequestContext,
    ) -> Result<OidcConfigId, ApplicationError> {
        let config = OidcConfig::new(
            cmd.name,
            cmd.client_id,
            cmd.issuer_url,
            cmd.audience,
            cmd.discovery_url,
            cmd.claim_mapping,
            ctx.clock(),
        )?;

        let config_id = config.id().clone();

        let uow = ctx.uow().await;
        let mut org = uow.organizations().find(&cmd.identifier).await?;
        org.add_draft_oidc_config(config)?;
        uow.organizations().save(org);

        Ok(config_id)
    }
}
