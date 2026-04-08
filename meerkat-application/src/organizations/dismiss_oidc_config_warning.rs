use std::sync::Arc;

use async_trait::async_trait;

use meerkat_domain::models::oidc_config::OidcConfigId;
use meerkat_domain::models::organization::OrganizationId;
use meerkat_domain::models::permission::OrgPermission;

use crate::behaviors::authorization::org_extensions;
use crate::context::RequestContext;
use crate::error::ApplicationError;
use mediator_rs::{Extensions, Request, Handler};
use crate::ports::oidc_config_warning_store::OidcConfigWarningStore;

pub struct DismissOidcConfigWarning {
    pub org_id: OrganizationId,
    pub config_id: OidcConfigId,
    pub warning_key: String,
}

impl Request for DismissOidcConfigWarning {
    type Output = ();

    fn extensions(&self) -> Extensions {
        org_extensions("DismissOidcConfigWarning", vec![OrgPermission::OrgManageOidc.into()])
    }
}

pub struct DismissOidcConfigWarningHandler {
    warning_store: Arc<dyn OidcConfigWarningStore>,
}

impl DismissOidcConfigWarningHandler {
    pub fn new(warning_store: Arc<dyn OidcConfigWarningStore>) -> Self {
        Self { warning_store }
    }
}

#[async_trait]
impl Handler<DismissOidcConfigWarning, ApplicationError, RequestContext> for DismissOidcConfigWarningHandler {
    async fn handle(
        &self,
        cmd: DismissOidcConfigWarning,
        _ctx: &RequestContext,
    ) -> Result<(), ApplicationError> {
        self.warning_store.dismiss(&cmd.config_id, &cmd.warning_key).await
    }
}
