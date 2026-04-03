use async_trait::async_trait;

use meerkat_domain::models::oidc_config::OidcConfigId;
use meerkat_domain::models::organization::OrganizationIdentifier;
use meerkat_domain::models::permission::OrgPermission;

use crate::behaviors::authorization::{RequestName, RequiredPermissions};
use crate::context::RequestContext;
use crate::error::ApplicationError;
use crate::extensions::Extensions;
use crate::mediator::{Request, Handler};

pub struct ActivateOidcConfig {
    pub org_identifier: OrganizationIdentifier,
    pub config_id: OidcConfigId,
}

impl Request for ActivateOidcConfig {
    type Output = ();

    fn extensions(&self) -> Extensions {
        let mut ext = Extensions::new();
        ext.insert(RequestName("ActivateOidcConfig".to_string()));
        ext.insert(RequiredPermissions(vec![OrgPermission::OrgManageOidc.into()]));
        ext
    }
}

pub struct ActivateOidcConfigHandler;

#[async_trait]
impl Handler<ActivateOidcConfig, ApplicationError, RequestContext> for ActivateOidcConfigHandler {
    async fn handle(
        &self,
        cmd: ActivateOidcConfig,
        ctx: &RequestContext,
    ) -> Result<(), ApplicationError> {
        let uow = ctx.uow().await;
        let mut org = uow.organizations().find(&cmd.org_identifier).await?;
        org.switch_active_oidc_config(&cmd.config_id)?;
        uow.organizations().save(org);
        Ok(())
    }
}
