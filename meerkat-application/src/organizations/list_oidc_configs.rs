use std::sync::Arc;

use async_trait::async_trait;

use meerkat_domain::models::organization::OrganizationId;
use meerkat_domain::models::permission::OrgPermission;

use crate::behaviors::authorization::org_extensions;
use crate::context::RequestContext;
use crate::error::ApplicationError;
use crate::extensions::Extensions;
use crate::mediator::{Request, Handler};
use crate::ports::oidc_config_read_store::{OidcConfigReadModel, OidcConfigReadStore};

pub struct ListOidcConfigs {
    pub org_id: OrganizationId,
}

impl Request for ListOidcConfigs {
    type Output = Vec<OidcConfigReadModel>;

    fn extensions(&self) -> Extensions {
        org_extensions("ListOidcConfigs", vec![OrgPermission::OrgManageOidc.into()])
    }
}

pub struct ListOidcConfigsHandler {
    oidc_config_read_store: Arc<dyn OidcConfigReadStore>,
}

impl ListOidcConfigsHandler {
    pub fn new(oidc_config_read_store: Arc<dyn OidcConfigReadStore>) -> Self {
        Self { oidc_config_read_store }
    }
}

#[async_trait]
impl Handler<ListOidcConfigs, ApplicationError, RequestContext> for ListOidcConfigsHandler {
    async fn handle(
        &self,
        cmd: ListOidcConfigs,
        _ctx: &RequestContext,
    ) -> Result<Vec<OidcConfigReadModel>, ApplicationError> {
        self.oidc_config_read_store
            .list_by_org_id(&cmd.org_id)
            .await
    }
}
