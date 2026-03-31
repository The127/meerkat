use std::sync::Arc;

use meerkat_application::context::{AppContext, RequestContext};
use meerkat_application::error::ApplicationError;
use meerkat_application::mediator::Mediator;
use meerkat_application::ports::health::HealthChecker;
use meerkat_application::ports::organization_read_store::OrganizationReadStore;

#[derive(Clone)]
pub struct AppState {
    pub health_checker: Arc<dyn HealthChecker>,
    pub mediator: Arc<Mediator<RequestContext, ApplicationError>>,
    pub context: Arc<AppContext>,
    pub org_read_store: Arc<dyn OrganizationReadStore>,
    pub base_domain: String,
    pub master_org_slug: String,
}
