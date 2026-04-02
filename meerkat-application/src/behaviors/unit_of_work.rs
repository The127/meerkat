use std::any::Any;

use async_trait::async_trait;

use crate::context::RequestContext;
use crate::error::ApplicationError;
use crate::extensions::Extensions;
use crate::mediator::{PipelineBehavior, PipelineNext};

pub struct UnitOfWorkBehavior;

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
                let mut uow = ctx.take_uow().await.expect("UoW was not scoped");
                uow.save_changes().await?;
                Ok(output)
            }
            Err(e) => {
                ctx.take_uow().await; // discard without saving
                Err(e)
            }
        }
    }
}
