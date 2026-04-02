use std::any::Any;
use std::sync::Arc;

use async_trait::async_trait;

use meerkat_domain::models::permission::EffectivePermission;

use crate::context::RequestContext;
use crate::error::ApplicationError;
use crate::extensions::Extensions;
use crate::mediator::{PipelineBehavior, PipelineNext};
use crate::ports::audit::{AuditEvent, AuditLogger, AuditOutcome};

pub struct RequiredPermissions(pub Vec<EffectivePermission>);
pub struct CommandName(pub String);

pub struct AuthorizationBehavior {
    audit_logger: Arc<dyn AuditLogger>,
}

impl AuthorizationBehavior {
    pub fn new(audit_logger: Arc<dyn AuditLogger>) -> Self {
        Self { audit_logger }
    }
}

#[async_trait]
impl PipelineBehavior<RequestContext, ApplicationError> for AuthorizationBehavior {
    async fn handle(
        &self,
        extensions: &Extensions,
        ctx: &RequestContext,
        next: PipelineNext<'_, ApplicationError>,
    ) -> Result<Box<dyn Any + Send + Sync>, ApplicationError> {
        let Some(RequiredPermissions(required)) = extensions.get::<RequiredPermissions>() else {
            return next.run().await;
        };

        if required.is_empty() {
            return next.run().await;
        }

        let action = extensions
            .get::<CommandName>()
            .map(|n| n.0.clone())
            .unwrap_or_else(|| "unknown".to_string());

        let auth = match ctx.auth() {
            Some(auth) => auth,
            None => {
                self.audit_logger.log(&AuditEvent {
                    sub: None,
                    org_id: None,
                    action: action.clone(),
                    required_permissions: required.clone(),
                    outcome: AuditOutcome::Unauthorized,
                }).await;
                return Err(ApplicationError::Unauthorized);
            }
        };

        for permission in required {
            if !auth.has_permission(permission.clone()) {
                self.audit_logger.log(&AuditEvent {
                    sub: Some(auth.sub.clone()),
                    org_id: Some(auth.org_id.clone()),
                    action: action.clone(),
                    required_permissions: required.clone(),
                    outcome: AuditOutcome::Denied,
                }).await;
                return Err(ApplicationError::Forbidden);
            }
        }

        self.audit_logger.log(&AuditEvent {
            sub: Some(auth.sub.clone()),
            org_id: Some(auth.org_id.clone()),
            action,
            required_permissions: required.clone(),
            outcome: AuditOutcome::Allowed,
        }).await;

        next.run().await
    }
}
