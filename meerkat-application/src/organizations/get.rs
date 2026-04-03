use async_trait::async_trait;

use meerkat_domain::models::organization::OrganizationIdentifier;

use crate::context::RequestContext;
use crate::error::ApplicationError;
use crate::mediator::{Request, Handler};
use crate::ports::organization_read_store::{OrganizationReadModel, OrganizationReadStore};

pub struct GetOrganization {
    pub identifier: OrganizationIdentifier,
}

impl Request for GetOrganization {
    type Output = OrganizationReadModel;
}

pub struct GetOrganizationHandler {
    org_read_store: std::sync::Arc<dyn OrganizationReadStore>,
}

impl GetOrganizationHandler {
    pub fn new(org_read_store: std::sync::Arc<dyn OrganizationReadStore>) -> Self {
        Self { org_read_store }
    }
}

#[async_trait]
impl Handler<GetOrganization, ApplicationError, RequestContext> for GetOrganizationHandler {
    async fn handle(
        &self,
        cmd: GetOrganization,
        _ctx: &RequestContext,
    ) -> Result<OrganizationReadModel, ApplicationError> {
        self.org_read_store
            .find(&cmd.identifier)
            .await?
            .ok_or(ApplicationError::NotFound)
    }
}
