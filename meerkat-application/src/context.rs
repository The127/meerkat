use std::ops::Deref;
use std::sync::Arc;

use tokio::sync::{Mutex, MutexGuard};

use meerkat_domain::ports::clock::Clock;

use crate::ports::error_observer::ErrorObserver;
use crate::ports::unit_of_work::{UnitOfWork, UnitOfWorkFactory};

/// RAII guard that holds the UoW mutex and derefs to `&dyn UnitOfWork`.
/// Can be held across `.await` points (tokio::sync::MutexGuard is Send).
pub struct ScopedUow<'a>(MutexGuard<'a, Option<Box<dyn UnitOfWork>>>);

impl Deref for ScopedUow<'_> {
    type Target = dyn UnitOfWork;

    fn deref(&self) -> &Self::Target {
        self.0
            .as_ref()
            .expect("no UoW scoped for this request")
            .as_ref()
    }
}

/// Shared, long-lived application services. Safe to share across requests.
pub struct AppContext {
    pub clock: Arc<dyn Clock>,
    pub uow_factory: Arc<dyn UnitOfWorkFactory>,
    pub error_observer: Arc<dyn ErrorObserver>,
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
        }
    }
}

/// Per-request context. Created fresh for each mediator dispatch.
pub struct RequestContext {
    pub app: Arc<AppContext>,
    scoped_uow: Mutex<Option<Box<dyn UnitOfWork>>>,
}

impl RequestContext {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self {
            app,
            scoped_uow: Mutex::new(None),
        }
    }

    pub fn clock(&self) -> &dyn Clock {
        self.app.clock.as_ref()
    }

    pub fn uow_factory(&self) -> &dyn UnitOfWorkFactory {
        self.app.uow_factory.as_ref()
    }

    pub async fn scope_uow(&self, uow: Box<dyn UnitOfWork>) {
        *self.scoped_uow.lock().await = Some(uow);
    }

    pub async fn take_uow(&self) -> Option<Box<dyn UnitOfWork>> {
        self.scoped_uow.lock().await.take()
    }

    pub async fn uow(&self) -> ScopedUow<'_> {
        ScopedUow(self.scoped_uow.lock().await)
    }
}

#[cfg(any(test, feature = "test-utils"))]
impl RequestContext {
    pub fn test() -> Self {
        let app = Arc::new(AppContext::new(
            Arc::new(meerkat_domain::ports::clock::MockClock::new(chrono::Utc::now())),
            Arc::new(crate::ports::unit_of_work::MockUnitOfWorkFactory::new()),
            Arc::new(crate::ports::error_observer::ErrorPipeline::new(vec![])),
        ));
        Self::new(app)
    }

    pub fn with_scoped_uow(self, uow: Box<dyn UnitOfWork>) -> Self {
        self.scoped_uow.try_lock().expect("mutex not yet contested during test setup").replace(uow);
        self
    }
}
