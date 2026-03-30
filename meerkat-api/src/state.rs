use std::sync::Arc;
use meerkat_domain::ports::health::HealthChecker;

#[derive(Clone)]
pub struct AppState {
    pub health_checker: Arc<dyn HealthChecker>,
}