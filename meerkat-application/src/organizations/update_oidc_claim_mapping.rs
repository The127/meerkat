use async_trait::async_trait;

use meerkat_domain::models::oidc_config::{ClaimMapping, OidcConfigId};
use meerkat_domain::models::organization::OrganizationIdentifier;
use meerkat_domain::models::permission::OrgPermission;

use crate::behaviors::authorization::org_extensions;
use crate::context::RequestContext;
use crate::error::ApplicationError;
use crate::extensions::Extensions;
use crate::mediator::{Request, Handler};

pub struct UpdateOidcClaimMapping {
    pub org_identifier: OrganizationIdentifier,
    pub config_id: OidcConfigId,
    pub claim_mapping: ClaimMapping,
}

impl Request for UpdateOidcClaimMapping {
    type Output = ();

    fn extensions(&self) -> Extensions {
        org_extensions("UpdateOidcClaimMapping", vec![OrgPermission::OrgManageOidc.into()])
    }
}

pub struct UpdateOidcClaimMappingHandler;

#[async_trait]
impl Handler<UpdateOidcClaimMapping, ApplicationError, RequestContext> for UpdateOidcClaimMappingHandler {
    async fn handle(
        &self,
        cmd: UpdateOidcClaimMapping,
        ctx: &RequestContext,
    ) -> Result<(), ApplicationError> {
        let uow = ctx.uow().await;
        let mut org = uow.organizations().find(&cmd.org_identifier).await?;
        org.update_oidc_config_claim_mapping(&cmd.config_id, cmd.claim_mapping)?;
        uow.organizations().save(org);
        Ok(())
    }
}
