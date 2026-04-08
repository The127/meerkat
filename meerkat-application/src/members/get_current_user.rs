use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;

use meerkat_domain::models::member::MemberId;
use meerkat_domain::models::permission::{EffectivePermission, OrgPermission, ProjectPermission};
use meerkat_domain::models::project::ProjectSlug;

use crate::context::RequestContext;
use crate::error::ApplicationError;
use mediator_rs::{Request, Handler};
use crate::ports::project_permission_read_store::ProjectPermissionReadStore;

pub struct GetCurrentUser;

#[derive(Debug, Clone)]
pub struct CurrentUserReadModel {
    pub member_id: MemberId,
    pub preferred_name: String,
    pub org_permissions: Vec<OrgPermission>,
    pub project_permissions: HashMap<ProjectSlug, Vec<ProjectPermission>>,
}

impl Request for GetCurrentUser {
    type Output = CurrentUserReadModel;
}

pub struct GetCurrentUserHandler {
    project_permission_store: Arc<dyn ProjectPermissionReadStore>,
}

impl GetCurrentUserHandler {
    pub fn new(project_permission_store: Arc<dyn ProjectPermissionReadStore>) -> Self {
        Self { project_permission_store }
    }
}

#[async_trait]
impl Handler<GetCurrentUser, ApplicationError, RequestContext> for GetCurrentUserHandler {
    async fn handle(
        &self,
        _cmd: GetCurrentUser,
        ctx: &RequestContext,
    ) -> Result<CurrentUserReadModel, ApplicationError> {
        let auth = ctx.auth().ok_or(ApplicationError::Unauthorized)?;

        let org_permissions: Vec<OrgPermission> = auth
            .permissions
            .iter()
            .filter_map(|p| match p {
                EffectivePermission::Org(op) => Some(op.clone()),
                _ => None,
            })
            .collect();

        let project_permissions = self
            .project_permission_store
            .get_all_member_permissions(&auth.member_id)
            .await?;

        Ok(CurrentUserReadModel {
            member_id: auth.member_id.clone(),
            preferred_name: auth.preferred_name.clone(),
            org_permissions,
            project_permissions,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::collections::{HashMap, HashSet};

    use mockall::predicate::*;

    use meerkat_domain::models::member::{MemberId, Sub};
    use meerkat_domain::models::org_role::OrgRole;
    use meerkat_domain::models::organization::OrganizationId;
    use meerkat_domain::models::permission::{EffectivePermission, OrgPermission, ProjectPermission};
    use meerkat_domain::models::project::ProjectSlug;

    use crate::auth_context::AuthContext;
    use crate::context::RequestContext;
    use mediator_rs::Handler;
    use crate::ports::project_permission_read_store::MockProjectPermissionReadStore;

    use super::*;

    fn test_auth_context() -> AuthContext {
        let member_id = MemberId::new();
        let mut permissions = HashSet::new();
        permissions.insert(EffectivePermission::Org(OrgPermission::OrgRename));
        permissions.insert(EffectivePermission::Org(OrgPermission::OrgCreateProject));

        AuthContext {
            sub: Sub::new("user-123").unwrap(),
            org_id: OrganizationId::new(),
            org_roles: vec![OrgRole::Admin],
            member_id,
            permissions,
            preferred_name: "Alice".to_string(),
        }
    }

    #[tokio::test]
    async fn given_authenticated_user_then_returns_identity_and_permissions() {
        // arrange
        let auth = test_auth_context();
        let member_id = auth.member_id.clone();

        let mut mock_store = MockProjectPermissionReadStore::new();
        let project_slug = ProjectSlug::new("my-project").unwrap();
        let mut expected_project_perms = HashMap::new();
        expected_project_perms.insert(
            project_slug.clone(),
            vec![ProjectPermission::ProjectRead, ProjectPermission::ProjectWrite],
        );
        let expected_clone = expected_project_perms.clone();

        mock_store
            .expect_get_all_member_permissions()
            .with(eq(member_id.clone()))
            .returning(move |_| Box::pin(std::future::ready(Ok(expected_clone.clone()))));

        let handler = GetCurrentUserHandler::new(Arc::new(mock_store));
        let ctx = RequestContext::test().with_auth(auth);

        // act
        let result = handler.handle(GetCurrentUser, &ctx).await.unwrap();

        // assert
        assert_eq!(result.member_id, member_id);
        assert_eq!(result.preferred_name, "Alice");
        assert_eq!(result.org_permissions.len(), 2);
        assert!(result.org_permissions.contains(&OrgPermission::OrgRename));
        assert!(result.org_permissions.contains(&OrgPermission::OrgCreateProject));

        let project_perms = result.project_permissions.get(&project_slug).unwrap();
        assert_eq!(project_perms.len(), 2);
        assert!(project_perms.contains(&ProjectPermission::ProjectRead));
        assert!(project_perms.contains(&ProjectPermission::ProjectWrite));
    }

    #[tokio::test]
    async fn given_no_auth_context_then_returns_unauthorized() {
        // arrange
        let mut mock_store = MockProjectPermissionReadStore::new();
        mock_store.expect_get_all_member_permissions().never();

        let handler = GetCurrentUserHandler::new(Arc::new(mock_store));
        let ctx = RequestContext::test();

        // act
        let result = handler.handle(GetCurrentUser, &ctx).await;

        // assert
        assert!(matches!(result, Err(ApplicationError::Unauthorized)));
    }
}
