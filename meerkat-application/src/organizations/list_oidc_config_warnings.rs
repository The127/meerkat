use std::sync::Arc;

use async_trait::async_trait;

use meerkat_domain::models::oidc_config::OidcConfigId;
use meerkat_domain::models::organization::OrganizationId;
use meerkat_domain::models::permission::OrgPermission;

use crate::behaviors::authorization::org_extensions;
use crate::context::RequestContext;
use crate::error::ApplicationError;
use crate::extensions::Extensions;
use crate::mediator::{Request, Handler};
use crate::ports::oidc_config_warning_store::{OidcConfigWarningReadModel, OidcConfigWarningStore};

pub struct ListOidcConfigWarnings {
    pub org_id: OrganizationId,
    pub config_id: OidcConfigId,
}

impl Request for ListOidcConfigWarnings {
    type Output = Vec<OidcConfigWarningReadModel>;

    fn extensions(&self) -> Extensions {
        org_extensions("ListOidcConfigWarnings", vec![OrgPermission::OrgManageOidc.into()])
    }
}

pub struct ListOidcConfigWarningsHandler {
    warning_store: Arc<dyn OidcConfigWarningStore>,
}

impl ListOidcConfigWarningsHandler {
    pub fn new(warning_store: Arc<dyn OidcConfigWarningStore>) -> Self {
        Self { warning_store }
    }
}

#[async_trait]
impl Handler<ListOidcConfigWarnings, ApplicationError, RequestContext> for ListOidcConfigWarningsHandler {
    async fn handle(
        &self,
        cmd: ListOidcConfigWarnings,
        _ctx: &RequestContext,
    ) -> Result<Vec<OidcConfigWarningReadModel>, ApplicationError> {
        self.warning_store.list_by_config(&cmd.config_id).await
    }
}
