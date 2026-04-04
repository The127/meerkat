use meerkat_domain::models::event::Event;

#[cfg_attr(any(test, feature = "test-utils"), mockall::automock)]
pub trait EventRepository: Send + Sync {
    fn add(&self, event: Event);
}

#[cfg(any(test, feature = "test-utils"))]
pub struct NoOpEventRepository;

#[cfg(any(test, feature = "test-utils"))]
impl EventRepository for NoOpEventRepository {
    fn add(&self, _event: Event) {}
}
