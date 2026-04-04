use meerkat_domain::models::event::Event;

use crate::error::ApplicationError;

#[async_trait::async_trait]
#[cfg_attr(any(test, feature = "test-utils"), mockall::automock)]
pub trait EventRepository: Send + Sync {
    async fn add(&self, event: &Event) -> Result<(), ApplicationError>;
}
