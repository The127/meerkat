use async_trait::async_trait;

use crate::error::ApplicationError;
use crate::ports::organization_repository::OrganizationRepository;
use crate::ports::project_member_repository::ProjectMemberRepository;
use crate::ports::project_repository::ProjectRepository;
use crate::ports::project_role_repository::ProjectRoleRepository;

#[async_trait]
pub trait UnitOfWork: Send + Sync {
    fn organizations(&self) -> &dyn OrganizationRepository;
    fn projects(&self) -> &dyn ProjectRepository;
    fn project_roles(&self) -> &dyn ProjectRoleRepository;
    fn project_members(&self) -> &dyn ProjectMemberRepository;
    async fn save_changes(&mut self) -> Result<(), ApplicationError>;
}

#[cfg_attr(any(test, feature = "test-utils"), mockall::automock)]
#[async_trait]
pub trait UnitOfWorkFactory: Send + Sync {
    async fn create(&self) -> Result<Box<dyn UnitOfWork>, ApplicationError>;
}

// UnitOfWork can't use #[automock] because it returns &dyn trait references.
#[cfg(any(test, feature = "test-utils"))]
pub struct MockUnitOfWork {
    org_repo: crate::ports::organization_repository::MockOrganizationRepository,
    project_repo: crate::ports::project_repository::MockProjectRepository,
    project_role_repo: crate::ports::project_role_repository::NoOpProjectRoleRepository,
    project_member_repo: crate::ports::project_member_repository::NoOpProjectMemberRepository,
    save_changes_result: Option<Result<(), ApplicationError>>,
}

#[cfg(any(test, feature = "test-utils"))]
impl Default for MockUnitOfWork {
    fn default() -> Self {
        Self {
            org_repo: crate::ports::organization_repository::MockOrganizationRepository::new(),
            project_repo: crate::ports::project_repository::MockProjectRepository::new(),
            project_role_repo: crate::ports::project_role_repository::NoOpProjectRoleRepository,
            project_member_repo: crate::ports::project_member_repository::NoOpProjectMemberRepository,
            save_changes_result: Some(Ok(())),
        }
    }
}

#[cfg(any(test, feature = "test-utils"))]
impl MockUnitOfWork {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_organization_repo(mut self, repo: crate::ports::organization_repository::MockOrganizationRepository) -> Self {
        self.org_repo = repo;
        self
    }

    pub fn with_project_repo(mut self, repo: crate::ports::project_repository::MockProjectRepository) -> Self {
        self.project_repo = repo;
        self
    }

    pub fn with_save_changes_result(mut self, result: Result<(), ApplicationError>) -> Self {
        self.save_changes_result = Some(result);
        self
    }
}

#[cfg(any(test, feature = "test-utils"))]
#[async_trait]
impl UnitOfWork for MockUnitOfWork {
    fn organizations(&self) -> &dyn OrganizationRepository {
        &self.org_repo
    }

    fn projects(&self) -> &dyn ProjectRepository {
        &self.project_repo
    }

    fn project_roles(&self) -> &dyn ProjectRoleRepository {
        &self.project_role_repo
    }

    fn project_members(&self) -> &dyn ProjectMemberRepository {
        &self.project_member_repo
    }

    async fn save_changes(&mut self) -> Result<(), ApplicationError> {
        self.save_changes_result
            .take()
            .unwrap_or(Ok(()))
    }
}
