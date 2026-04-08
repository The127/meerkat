use std::any::Any;
use std::sync::Arc;

use async_trait::async_trait;

use crate::context::RequestContext;
use crate::error::ApplicationError;
use crate::events::EventDispatcher;
use mediator_rs::{Extensions, PipelineBehavior, PipelineNext};

pub struct UnitOfWorkBehavior {
    event_dispatcher: Arc<EventDispatcher>,
}

impl UnitOfWorkBehavior {
    pub fn new(event_dispatcher: Arc<EventDispatcher>) -> Self {
        Self { event_dispatcher }
    }
}

#[async_trait]
impl PipelineBehavior<RequestContext, ApplicationError> for UnitOfWorkBehavior {
    async fn handle(
        &self,
        _extensions: &Extensions,
        ctx: &RequestContext,
        next: PipelineNext<'_, ApplicationError>,
    ) -> Result<Box<dyn Any + Send + Sync>, ApplicationError> {
        let uow = ctx.uow_factory().create().await?;
        ctx.scope_uow(uow).await;

        let result = next.run().await;

        match result {
            Ok(output) => {
                let events = ctx.drain_events().await;
                self.event_dispatcher.dispatch_all(events, ctx).await?;

                let mut uow = ctx.take_uow().await.expect("UoW was not scoped");
                uow.save_changes().await?;
                Ok(output)
            }
            Err(e) => {
                ctx.take_uow().await;
                Err(e)
            }
        }
    }
}
