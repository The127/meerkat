use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};

use meerkat_domain::models::member::MemberId;
use meerkat_domain::models::org_role::OrgRole;
use meerkat_domain::models::organization::OrganizationId;
use meerkat_domain::models::permission::{OrgPermission, ProjectPermission};
use meerkat_domain::models::project::ProjectSlug;

use crate::behaviors::authorization::org_extensions;
use crate::context::RequestContext;
use crate::error::ApplicationError;
use crate::extensions::Extensions;
use crate::mediator::{Handler, Request};
use crate::ports::member_read_store::MemberReadStore;
use crate::ports::project_member_read_store::{MemberProjectAccessReadModel, ProjectMemberReadStore};

pub struct GetMemberAccess {
    pub member_id: MemberId,
    pub org_id: OrganizationId,
}

impl Request for GetMemberAccess {
    type Output = MemberAccessResult;

    fn extensions(&self) -> Extensions {
        org_extensions("GetMemberAccess", vec![OrgPermission::OrgManageMembers.into()])
    }
}

#[derive(Debug, Clone)]
pub struct MemberAccessResult {
    pub id: MemberId,
    pub preferred_name: String,
    pub sub: String,
    pub created_at: DateTime<Utc>,
    pub last_seen: DateTime<Utc>,
    pub org_access: OrgAccessResult,
    pub project_access: Vec<ProjectAccessResult>,
}

#[derive(Debug, Clone)]
pub struct OrgAccessResult {
    pub roles: Vec<OrgRole>,
    pub permissions: Vec<OrgPermission>,
}

#[derive(Debug, Clone)]
pub struct ProjectAccessResult {
    pub project_name: String,
    pub project_slug: ProjectSlug,
    pub roles: Vec<ProjectRoleAccessResult>,
}

#[derive(Debug, Clone)]
pub struct ProjectRoleAccessResult {
    pub role_name: String,
    pub permissions: Vec<ProjectPermission>,
}

pub struct GetMemberAccessHandler {
    member_read_store: Arc<dyn MemberReadStore>,
    project_member_read_store: Arc<dyn ProjectMemberReadStore>,
}

impl GetMemberAccessHandler {
    pub fn new(
        member_read_store: Arc<dyn MemberReadStore>,
        project_member_read_store: Arc<dyn ProjectMemberReadStore>,
    ) -> Self {
        Self {
            member_read_store,
            project_member_read_store,
        }
    }
}

fn group_by_project(rows: Vec<MemberProjectAccessReadModel>) -> Vec<ProjectAccessResult> {
    let mut project_access: Vec<ProjectAccessResult> = Vec::new();
    for row in rows {
        let slug_str = row.project_slug.as_str().to_string();
        if let Some(existing) = project_access.iter_mut().find(|p| p.project_slug.as_str() == slug_str) {
            existing.roles.push(ProjectRoleAccessResult {
                role_name: row.role_name,
                permissions: row.role_permissions,
            });
        } else {
            project_access.push(ProjectAccessResult {
                project_name: row.project_name,
                project_slug: row.project_slug,
                roles: vec![ProjectRoleAccessResult {
                    role_name: row.role_name,
                    permissions: row.role_permissions,
                }],
            });
        }
    }
    project_access
}

#[async_trait]
impl Handler<GetMemberAccess, ApplicationError, RequestContext> for GetMemberAccessHandler {
    async fn handle(
        &self,
        cmd: GetMemberAccess,
        _ctx: &RequestContext,
    ) -> Result<MemberAccessResult, ApplicationError> {
        let member = self
            .member_read_store
            .find_member_for_access(&cmd.member_id, &cmd.org_id)
            .await?
            .ok_or(ApplicationError::NotFound)?;

        let project_rows = self
            .project_member_read_store
            .list_access_by_member(&cmd.member_id, &cmd.org_id)
            .await?;

        let org_permissions: Vec<OrgPermission> = member
            .org_roles
            .iter()
            .flat_map(|r| r.permissions())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();

        Ok(MemberAccessResult {
            id: member.id,
            preferred_name: member.preferred_name,
            sub: member.sub,
            created_at: member.created_at,
            last_seen: member.last_seen,
            org_access: OrgAccessResult {
                roles: member.org_roles,
                permissions: org_permissions,
            },
            project_access: group_by_project(project_rows),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ports::member_read_store::{MemberReadModel, MockMemberReadStore};
    use crate::ports::project_member_read_store::{MemberProjectAccessReadModel, MockProjectMemberReadStore};
    use meerkat_domain::models::permission::ProjectPermission;

    fn test_member() -> MemberReadModel {
        MemberReadModel {
            id: MemberId::new(),
            sub: "user-123".to_string(),
            preferred_name: "Alice".to_string(),
            org_roles: vec![OrgRole::Owner],
            created_at: Utc::now(),
            last_seen: Utc::now(),
        }
    }

    #[tokio::test]
    async fn given_existing_member_then_returns_access() {
        // arrange
        let org_id = OrganizationId::new();
        let member = test_member();
        let member_id = member.id.clone();

        let mut member_store = MockMemberReadStore::new();
        let returned_member = member.clone();
        member_store
            .expect_find_member_for_access()
            .withf({
                let member_id = member_id.clone();
                let org_id = org_id.clone();
                move |mid, oid| *mid == member_id && *oid == org_id
            })
            .returning(move |_, _| Box::pin(std::future::ready(Ok(Some(returned_member.clone())))));

        let mut project_member_store = MockProjectMemberReadStore::new();
        project_member_store
            .expect_list_access_by_member()
            .withf({
                let member_id = member_id.clone();
                let org_id = org_id.clone();
                move |mid, oid| *mid == member_id && *oid == org_id
            })
            .returning(|_, _| Box::pin(std::future::ready(Ok(vec![
                MemberProjectAccessReadModel {
                    project_name: "My Project".to_string(),
                    project_slug: ProjectSlug::new("my-project".to_string()).unwrap(),
                    role_name: "Editor".to_string(),
                    role_permissions: vec![ProjectPermission::ProjectRead, ProjectPermission::ProjectWrite],
                },
            ]))));

        let handler = GetMemberAccessHandler::new(
            Arc::new(member_store),
            Arc::new(project_member_store),
        );

        // act
        let result = handler
            .handle(
                GetMemberAccess { member_id, org_id },
                &RequestContext::test(),
            )
            .await
            .unwrap();

        // assert
        assert_eq!(result.preferred_name, "Alice");
        assert_eq!(result.org_access.roles, vec![OrgRole::Owner]);
        assert!(!result.org_access.permissions.is_empty());
        assert_eq!(result.project_access.len(), 1);
        assert_eq!(result.project_access[0].project_name, "My Project");
        assert_eq!(result.project_access[0].roles.len(), 1);
        assert_eq!(result.project_access[0].roles[0].role_name, "Editor");
    }

    #[tokio::test]
    async fn given_nonexistent_member_then_returns_not_found() {
        // arrange
        let org_id = OrganizationId::new();
        let member_id = MemberId::new();

        let mut member_store = MockMemberReadStore::new();
        member_store
            .expect_find_member_for_access()
            .returning(|_, _| Box::pin(std::future::ready(Ok(None))));

        let mut project_member_store = MockProjectMemberReadStore::new();
        project_member_store.expect_list_access_by_member().never();

        let handler = GetMemberAccessHandler::new(
            Arc::new(member_store),
            Arc::new(project_member_store),
        );

        // act
        let result = handler
            .handle(
                GetMemberAccess { member_id, org_id },
                &RequestContext::test(),
            )
            .await;

        // assert
        assert!(matches!(result, Err(ApplicationError::NotFound)));
    }

    #[tokio::test]
    async fn given_member_with_multiple_roles_per_project_then_groups_by_project() {
        // arrange
        let org_id = OrganizationId::new();
        let member = test_member();
        let member_id = member.id.clone();

        let mut member_store = MockMemberReadStore::new();
        let returned_member = member.clone();
        member_store
            .expect_find_member_for_access()
            .returning(move |_, _| Box::pin(std::future::ready(Ok(Some(returned_member.clone())))));

        let mut project_member_store = MockProjectMemberReadStore::new();
        project_member_store
            .expect_list_access_by_member()
            .returning(|_, _| Box::pin(std::future::ready(Ok(vec![
                MemberProjectAccessReadModel {
                    project_name: "My Project".to_string(),
                    project_slug: ProjectSlug::new("my-project".to_string()).unwrap(),
                    role_name: "Editor".to_string(),
                    role_permissions: vec![ProjectPermission::ProjectRead, ProjectPermission::ProjectWrite],
                },
                MemberProjectAccessReadModel {
                    project_name: "My Project".to_string(),
                    project_slug: ProjectSlug::new("my-project".to_string()).unwrap(),
                    role_name: "Admin".to_string(),
                    role_permissions: vec![ProjectPermission::ProjectDelete],
                },
            ]))));

        let handler = GetMemberAccessHandler::new(
            Arc::new(member_store),
            Arc::new(project_member_store),
        );

        // act
        let result = handler
            .handle(
                GetMemberAccess { member_id, org_id },
                &RequestContext::test(),
            )
            .await
            .unwrap();

        // assert
        assert_eq!(result.project_access.len(), 1);
        assert_eq!(result.project_access[0].roles.len(), 2);
    }
}
