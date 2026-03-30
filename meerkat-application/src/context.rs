use std::sync::Arc;

use meerkat_domain::ports::clock::Clock;

use crate::ports::error_observer::ErrorObserver;
use crate::ports::unit_of_work::UnitOfWorkFactory;

pub struct AppContext {
    pub clock: Arc<dyn Clock>,
    pub uow_factory: Arc<dyn UnitOfWorkFactory>,
    pub error_observer: Arc<dyn ErrorObserver>,
}
