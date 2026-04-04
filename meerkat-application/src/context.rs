use std::ops::Deref;
use std::sync::Arc;

use tokio::sync::{Mutex, MutexGuard};

use crate::auth_context::AuthContext;
use crate::events::DomainEvent;
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
    pub uow_factory: Arc<dyn UnitOfWorkFactory>,
    pub error_observer: Arc<dyn ErrorObserver>,
}

impl AppContext {
    pub fn new(
        uow_factory: Arc<dyn UnitOfWorkFactory>,
        error_observer: Arc<dyn ErrorObserver>,
    ) -> Self {
        Self {
            uow_factory,
            error_observer,
        }
    }
}

/// Per-request context. Created fresh for each mediator dispatch.
pub struct RequestContext {
    pub app: Arc<AppContext>,
    auth: Option<AuthContext>,
    scoped_uow: Mutex<Option<Box<dyn UnitOfWork>>>,
    events: Mutex<Vec<DomainEvent>>,
}

impl RequestContext {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self {
            app,
            auth: None,
            scoped_uow: Mutex::new(None),
            events: Mutex::new(Vec::new()),
        }
    }

    pub fn with_auth(mut self, auth: AuthContext) -> Self {
        self.auth = Some(auth);
        self
    }

    pub fn auth(&self) -> Option<&AuthContext> {
        self.auth.as_ref()
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

    pub async fn raise(&self, event: DomainEvent) {
        self.events.lock().await.push(event);
    }

    pub async fn drain_events(&self) -> Vec<DomainEvent> {
        std::mem::take(&mut *self.events.lock().await)
    }
}

#[cfg(any(test, feature = "test-utils"))]
impl RequestContext {
    pub fn test() -> Self {
        let app = Arc::new(AppContext::new(
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
