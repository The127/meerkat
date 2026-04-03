use async_trait::async_trait;

use meerkat_domain::models::organization::OrganizationId;

use crate::context::RequestContext;
use crate::error::ApplicationError;
use crate::mediator::{Request, Handler};
use crate::ports::oidc_config_read_store::{OidcConfigReadModel, OidcConfigReadStore};

pub struct GetOidcConfig {
    pub org_id: OrganizationId,
}

impl Request for GetOidcConfig {
    type Output = OidcConfigReadModel;
}

pub struct GetOidcConfigHandler {
    oidc_config_read_store: std::sync::Arc<dyn OidcConfigReadStore>,
}

impl GetOidcConfigHandler {
    pub fn new(oidc_config_read_store: std::sync::Arc<dyn OidcConfigReadStore>) -> Self {
        Self { oidc_config_read_store }
    }
}

#[async_trait]
impl Handler<GetOidcConfig, ApplicationError, RequestContext> for GetOidcConfigHandler {
    async fn handle(
        &self,
        cmd: GetOidcConfig,
        _ctx: &RequestContext,
    ) -> Result<OidcConfigReadModel, ApplicationError> {
        self.oidc_config_read_store
            .find_active_by_org_id(&cmd.org_id)
            .await
    }
}
