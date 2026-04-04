pub mod ingest;

use std::sync::Arc;

use async_trait::async_trait;

use meerkat_domain::models::issue::IssueId;
use meerkat_domain::models::project::ProjectId;

use crate::context::RequestContext;
use crate::error::ApplicationError;

#[derive(Debug, Clone)]
pub enum DomainEvent {
    ProjectCreated { project_id: ProjectId },
    EventRecorded { issue_id: IssueId },
}

#[async_trait]
pub trait DomainEventHandler: Send + Sync {
    async fn handle(&self, event: &DomainEvent, ctx: &RequestContext) -> Result<(), ApplicationError>;
}

#[derive(Default)]
pub struct EventDispatcher {
    handlers: Vec<Arc<dyn DomainEventHandler>>,
}

impl EventDispatcher {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, handler: Arc<dyn DomainEventHandler>) {
        self.handlers.push(handler);
    }

    pub async fn dispatch_all(
        &self,
        events: Vec<DomainEvent>,
        ctx: &RequestContext,
    ) -> Result<(), ApplicationError> {
        for event in &events {
            for handler in &self.handlers {
                handler.handle(event, ctx).await?;
            }
        }
        Ok(())
    }
}
