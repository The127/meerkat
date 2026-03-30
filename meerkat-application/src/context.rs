use std::sync::{Arc, Mutex};

use meerkat_domain::ports::clock::Clock;

use crate::ports::error_observer::ErrorObserver;
use crate::ports::unit_of_work::{UnitOfWork, UnitOfWorkFactory};

pub struct AppContext {
    pub clock: Arc<dyn Clock>,
    pub uow_factory: Arc<dyn UnitOfWorkFactory>,
    pub error_observer: Arc<dyn ErrorObserver>,
    scoped_uow: Mutex<Option<Box<dyn UnitOfWork>>>,
}

impl AppContext {
    pub fn new(
        clock: Arc<dyn Clock>,
        uow_factory: Arc<dyn UnitOfWorkFactory>,
        error_observer: Arc<dyn ErrorObserver>,
    ) -> Self {
        Self {
            clock,
            uow_factory,
            error_observer,
            scoped_uow: Mutex::new(None),
        }
    }

    pub fn scope_uow(&self, uow: Box<dyn UnitOfWork>) {
        *self.scoped_uow.lock().unwrap() = Some(uow);
    }

    pub fn take_uow(&self) -> Option<Box<dyn UnitOfWork>> {
        self.scoped_uow.lock().unwrap().take()
    }

    pub fn with_uow<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&dyn UnitOfWork) -> R,
    {
        let guard = self.scoped_uow.lock().unwrap();
        let uow = guard.as_ref().expect("no UoW scoped for this request");
        f(uow.as_ref())
    }
}

#[cfg(any(test, feature = "test-utils"))]
impl AppContext {
    /// Creates a test context with no-op defaults.
    /// Override individual fields with `with_clock`, `with_uow`, etc.
    pub fn test() -> Self {
        Self::new(
            Arc::new(meerkat_domain::ports::clock::MockClock::new(chrono::Utc::now())),
            Arc::new(crate::ports::unit_of_work::MockUnitOfWorkFactory::new()),
            Arc::new(crate::ports::error_observer::ErrorPipeline::new(vec![])),
        )
    }

    pub fn with_clock(mut self, clock: Arc<dyn Clock>) -> Self {
        self.clock = clock;
        self
    }

    pub fn with_scoped_uow(self, uow: Box<dyn UnitOfWork>) -> Self {
        self.scope_uow(uow);
        self
    }
}
