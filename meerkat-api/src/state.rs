use std::sync::Arc;

use meerkat_application::context::AppContext;
use meerkat_application::error::ApplicationError;
use meerkat_application::mediator::Mediator;
use meerkat_application::ports::health::HealthChecker;

#[derive(Clone)]
pub struct AppState {
    pub health_checker: Arc<dyn HealthChecker>,
    pub mediator: Arc<Mediator<AppContext, ApplicationError>>,
    pub context: Arc<AppContext>,
}