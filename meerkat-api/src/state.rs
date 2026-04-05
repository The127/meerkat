use std::sync::Arc;

use meerkat_application::context::{AppContext, RequestContext};
use meerkat_application::error::ApplicationError;
use meerkat_application::mediator::Mediator;
use meerkat_application::ports::health::HealthChecker;
use meerkat_application::ports::jwks_provider::JwksProvider;
use meerkat_application::ports::member_repository::MemberRepository;
use meerkat_application::ports::oidc_config_warning_store::OidcConfigWarningStore;
use meerkat_application::ports::oidc_discovery_provider::OidcDiscoveryProvider;
use meerkat_application::ports::oidc_config_read_store::OidcConfigReadStore;
use meerkat_application::ports::organization_read_store::OrganizationReadStore;
use meerkat_application::ports::project_key_read_store::ProjectKeyReadStore;

#[derive(Clone)]
pub struct AuthState {
    pub oidc_config_read_store: Arc<dyn OidcConfigReadStore>,
    pub jwks_provider: Arc<dyn JwksProvider>,
    pub oidc_discovery_provider: Arc<dyn OidcDiscoveryProvider>,
    pub member_repository: Arc<dyn MemberRepository>,
    pub warning_store: Arc<dyn OidcConfigWarningStore>,
}

#[derive(Clone)]
pub struct TenantState {
    pub org_read_store: Arc<dyn OrganizationReadStore>,
    pub base_domain: String,
    pub master_org_slug: String,
}

#[derive(Clone)]
pub struct AppState {
    pub health_checker: Arc<dyn HealthChecker>,
    pub mediator: Arc<Mediator<RequestContext, ApplicationError>>,
    pub context: Arc<AppContext>,
    pub auth: AuthState,
    pub tenant: TenantState,
    pub project_key_read_store: Arc<dyn ProjectKeyReadStore>,
    pub auth_enabled: bool,
}
