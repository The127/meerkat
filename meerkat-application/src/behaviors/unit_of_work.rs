use std::any::Any;

use async_trait::async_trait;

use crate::context::RequestContext;
use crate::error::ApplicationError;
use crate::mediator::{PipelineBehavior, PipelineNext};

pub struct UnitOfWorkBehavior;

#[async_trait]
impl PipelineBehavior<RequestContext, ApplicationError> for UnitOfWorkBehavior {
    async fn handle(
        &self,
        ctx: &RequestContext,
        next: PipelineNext<'_, ApplicationError>,
    ) -> Result<Box<dyn Any + Send>, ApplicationError> {
        let uow = ctx.uow_factory().create().await?;
        ctx.scope_uow(uow);

        let result = next.run().await;

        match result {
            Ok(output) => {
                let mut uow = ctx.take_uow().expect("UoW was not scoped");
                uow.save_changes().await?;
                Ok(output)
            }
            Err(e) => {
                ctx.take_uow(); // discard without saving
                Err(e)
            }
        }
    }
}
