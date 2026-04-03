use std::any::Any;
use std::sync::Arc;

use async_trait::async_trait;

use meerkat_domain::models::permission::EffectivePermission;
use meerkat_domain::models::project::ProjectIdentifier;

use crate::context::RequestContext;
use crate::error::ApplicationError;
use crate::extensions::Extensions;
use crate::mediator::{PipelineBehavior, PipelineNext};
use crate::ports::audit::{AuditEvent, AuditLogger, AuditOutcome};
use crate::ports::project_permission_read_store::ProjectPermissionReadStore;
use crate::ports::project_read_store::ProjectReadStore;

pub struct RequiredPermissions(pub Vec<EffectivePermission>);
pub struct ProjectContext(pub ProjectIdentifier);

#[cfg(any(test, feature = "test-utils"))]
pub mod testing {
    use std::sync::Arc;
    use async_trait::async_trait;
    use tokio::sync::Mutex;
    use crate::ports::audit::{AuditEvent, AuditLogger};

    pub struct CapturingAuditLogger {
        events: Mutex<Vec<AuditEvent>>,
    }

    impl CapturingAuditLogger {
        pub fn new() -> Arc<Self> {
            Arc::new(Self { events: Mutex::new(vec![]) })
        }

        pub async fn events(&self) -> Vec<AuditEvent> {
            self.events.lock().await.clone()
        }
    }

    #[async_trait]
    impl AuditLogger for CapturingAuditLogger {
        async fn log(&self, event: &AuditEvent) {
            self.events.lock().await.push(event.clone());
        }
    }
}

pub struct RequestName(pub String);

pub struct AuthorizationBehavior {
    audit_logger: Arc<dyn AuditLogger>,
    project_permission_store: Arc<dyn ProjectPermissionReadStore>,
    project_read_store: Arc<dyn ProjectReadStore>,
}

impl AuthorizationBehavior {
    pub fn new(
        audit_logger: Arc<dyn AuditLogger>,
        project_permission_store: Arc<dyn ProjectPermissionReadStore>,
        project_read_store: Arc<dyn ProjectReadStore>,
    ) -> Self {
        Self { audit_logger, project_permission_store, project_read_store }
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
            .get::<RequestName>()
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

        let mut effective = auth.permissions.clone();

        if let Some(ProjectContext(identifier)) = extensions.get::<ProjectContext>() {
            let project_id = match identifier {
                ProjectIdentifier::Id(id) => id.clone(),
                ProjectIdentifier::Slug(org_id, slug) => {
                    let project = self.project_read_store
                        .find_by_slug(org_id, slug)
                        .await
                        .map_err(|_| ApplicationError::Internal("failed to resolve project".to_string()))?
                        .ok_or(ApplicationError::NotFound)?;
                    project.id
                }
            };

            let project_perms = self.project_permission_store
                .get_member_permissions(&auth.member_id, &project_id)
                .await
                .map_err(|_| ApplicationError::Internal("failed to load project permissions".to_string()))?;

            for p in project_perms {
                effective.insert(EffectivePermission::Project(p));
            }
        }

        for permission in required {
            if !effective.contains(permission) {
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

#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use std::sync::Arc;

    use async_trait::async_trait;

    use meerkat_domain::models::member::{MemberId, Sub};
    use meerkat_domain::models::organization::OrganizationId;
    use meerkat_domain::models::permission::{OrgPermission, ProjectPermission};

    use crate::auth_context::AuthContext;
    use crate::behaviors::authorization::testing::CapturingAuditLogger;
    use crate::context::RequestContext;
    use crate::error::ApplicationError;
    use crate::extensions::Extensions;
    use crate::mediator::{Request, Handler, Mediator};
    use crate::ports::audit::AuditOutcome;

    use super::*;

    struct NoAuthCommand;
    impl Request for NoAuthCommand {
        type Output = String;
    }

    struct ProtectedCommand;
    impl Request for ProtectedCommand {
        type Output = String;
        fn extensions(&self) -> Extensions {
            let mut ext = Extensions::new();
            ext.insert(RequestName("ProtectedCommand".to_string()));
            ext.insert(RequiredPermissions(vec![OrgPermission::OrgRename.into()]));
            ext
        }
    }

    struct EchoHandler;

    #[async_trait]
    impl Handler<NoAuthCommand, ApplicationError, RequestContext> for EchoHandler {
        async fn handle(&self, _cmd: NoAuthCommand, _ctx: &RequestContext) -> Result<String, ApplicationError> {
            Ok("ok".to_string())
        }
    }

    #[async_trait]
    impl Handler<ProtectedCommand, ApplicationError, RequestContext> for EchoHandler {
        async fn handle(&self, _cmd: ProtectedCommand, _ctx: &RequestContext) -> Result<String, ApplicationError> {
            Ok("ok".to_string())
        }
    }

    fn test_auth(permissions: Vec<EffectivePermission>) -> AuthContext {
        AuthContext {
            sub: Sub::new("test-user").unwrap(),
            org_id: OrganizationId::new(),
            org_roles: vec![],
            member_id: MemberId::new(),
            permissions: HashSet::from_iter(permissions),
        }
    }

    fn build_mediator(audit_logger: Arc<dyn crate::ports::audit::AuditLogger>) -> Mediator<RequestContext, ApplicationError> {
        let mut perm_store = crate::ports::project_permission_read_store::MockProjectPermissionReadStore::new();
        perm_store.expect_get_member_permissions().returning(|_, _| Box::pin(async { Ok(vec![]) }));

        let mut project_read_store = crate::ports::project_read_store::MockProjectReadStore::new();
        project_read_store.expect_find_by_slug().returning(|_, _| Box::pin(async { Ok(None) }));

        let mut mediator = Mediator::new();
        mediator.add_behavior(Arc::new(AuthorizationBehavior::new(audit_logger, Arc::new(perm_store), Arc::new(project_read_store))));
        mediator.register::<NoAuthCommand, _>(EchoHandler);
        mediator.register::<ProtectedCommand, _>(EchoHandler);
        mediator
    }

    #[tokio::test]
    async fn given_command_without_auth_requirement_then_passes_through() {
        // arrange
        let logger = CapturingAuditLogger::new();
        let mediator = build_mediator(logger.clone());
        let ctx = RequestContext::test();

        // act
        let result = mediator.dispatch(NoAuthCommand, &ctx).await;

        // assert
        assert_eq!(result.unwrap(), "ok");
        assert!(logger.events().await.is_empty());
    }

    #[tokio::test]
    async fn given_authenticated_user_with_permission_then_succeeds() {
        // arrange
        let logger = CapturingAuditLogger::new();
        let mediator = build_mediator(logger.clone());
        let ctx = RequestContext::test()
            .with_auth(test_auth(vec![OrgPermission::OrgRename.into()]));

        // act
        let result = mediator.dispatch(ProtectedCommand, &ctx).await;

        // assert
        assert_eq!(result.unwrap(), "ok");
        let events = logger.events().await;
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].action, "ProtectedCommand");
        assert!(matches!(events[0].outcome, AuditOutcome::Allowed));
    }

    #[tokio::test]
    async fn given_no_auth_context_then_returns_unauthorized() {
        // arrange
        let logger = CapturingAuditLogger::new();
        let mediator = build_mediator(logger.clone());
        let ctx = RequestContext::test();

        // act
        let result = mediator.dispatch(ProtectedCommand, &ctx).await;

        // assert
        match result {
            Err(crate::mediator::MediatorError::HandlerError(ApplicationError::Unauthorized)) => (),
            other => panic!("Expected Unauthorized, got {:?}", other),
        }
        let events = logger.events().await;
        assert_eq!(events.len(), 1);
        assert!(matches!(events[0].outcome, AuditOutcome::Unauthorized));
    }

    #[tokio::test]
    async fn given_authenticated_user_without_permission_then_returns_forbidden() {
        // arrange
        let logger = CapturingAuditLogger::new();
        let mediator = build_mediator(logger.clone());
        let ctx = RequestContext::test()
            .with_auth(test_auth(vec![ProjectPermission::ProjectRead.into()]));

        // act
        let result = mediator.dispatch(ProtectedCommand, &ctx).await;

        // assert
        match result {
            Err(crate::mediator::MediatorError::HandlerError(ApplicationError::Forbidden)) => (),
            other => panic!("Expected Forbidden, got {:?}", other),
        }
        let events = logger.events().await;
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].action, "ProtectedCommand");
        assert!(matches!(events[0].outcome, AuditOutcome::Denied));
        assert!(events[0].sub.is_some());
    }
}
