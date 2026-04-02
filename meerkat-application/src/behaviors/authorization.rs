use std::any::Any;

use async_trait::async_trait;

use meerkat_domain::models::permission::EffectivePermission;

use crate::context::RequestContext;
use crate::error::ApplicationError;
use crate::extensions::Extensions;
use crate::mediator::{PipelineBehavior, PipelineNext};

pub struct RequiredPermissions(pub Vec<EffectivePermission>);

pub struct AuthorizationBehavior;

#[async_trait]
impl PipelineBehavior<RequestContext, ApplicationError> for AuthorizationBehavior {
    async fn handle(
        &self,
        extensions: &Extensions,
        ctx: &RequestContext,
        next: PipelineNext<'_, ApplicationError>,
    ) -> Result<Box<dyn Any + Send + Sync>, ApplicationError> {
        if let Some(RequiredPermissions(required)) = extensions.get::<RequiredPermissions>()
            && !required.is_empty()
        {
            let auth = ctx.auth().ok_or(ApplicationError::Unauthorized)?;

            for permission in required {
                if !auth.has_permission(permission.clone()) {
                    return Err(ApplicationError::Forbidden);
                }
            }
        }

        next.run().await
    }
}
