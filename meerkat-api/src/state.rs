use std::sync::Arc;

use meerkat_application::context::{AppContext, RequestContext};
use meerkat_application::error::ApplicationError;
use meerkat_application::mediator::Mediator;
use meerkat_application::ports::health::HealthChecker;
use meerkat_application::ports::jwks_provider::JwksProvider;
use meerkat_application::ports::oidc_discovery_provider::OidcDiscoveryProvider;
use meerkat_application::ports::oidc_config_read_store::OidcConfigReadStore;
use meerkat_application::ports::organization_read_store::OrganizationReadStore;

#[derive(Clone)]
pub struct AppState {
    pub health_checker: Arc<dyn HealthChecker>,
    pub mediator: Arc<Mediator<RequestContext, ApplicationError>>,
    pub context: Arc<AppContext>,
    pub org_read_store: Arc<dyn OrganizationReadStore>,
    pub oidc_config_read_store: Arc<dyn OidcConfigReadStore>,
    pub jwks_provider: Arc<dyn JwksProvider>,
    pub oidc_discovery_provider: Arc<dyn OidcDiscoveryProvider>,
    pub base_domain: String,
    pub master_org_slug: String,
    pub auth_enabled: bool,
}
