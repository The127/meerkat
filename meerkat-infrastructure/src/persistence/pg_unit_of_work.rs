use std::sync::Arc;

use async_trait::async_trait;
use sqlx::PgPool;

use meerkat_application::error::ApplicationError;
use meerkat_application::ports::organization_repository::OrganizationRepository;
use meerkat_application::ports::project_key_repository::ProjectKeyRepository;
use meerkat_application::ports::project_member_repository::ProjectMemberRepository;
use meerkat_application::ports::project_repository::ProjectRepository;
use meerkat_application::ports::project_role_repository::ProjectRoleRepository;
use meerkat_application::ports::unit_of_work::{UnitOfWork, UnitOfWorkFactory};
use meerkat_domain::ports::clock::Clock;

use crate::persistence::error::map_sqlx_error;
use crate::persistence::organization_persistence::OrganizationPersistence;
use crate::persistence::pg_organization_repository::{OrgEntry, PgOrganizationRepository};
use crate::persistence::pg_project_key_repository::{PgProjectKeyRepository, ProjectKeyEntry};
use crate::persistence::pg_project_member_repository::PgProjectMemberRepository;
use crate::persistence::pg_project_repository::{PgProjectRepository, ProjectEntry};
use crate::persistence::pg_project_role_repository::PgProjectRoleRepository;
use crate::persistence::project_key_persistence::ProjectKeyPersistence;
use crate::persistence::project_member_persistence::ProjectMemberPersistence;
use crate::persistence::project_persistence::ProjectPersistence;
use crate::persistence::project_role_persistence::ProjectRolePersistence;

pub struct PgUnitOfWork {
    pool: PgPool,
    clock: Arc<dyn Clock>,
    org_repo: PgOrganizationRepository,
    project_repo: PgProjectRepository,
    project_key_repo: PgProjectKeyRepository,
    project_role_repo: PgProjectRoleRepository,
    project_member_repo: PgProjectMemberRepository,
}

impl PgUnitOfWork {
    pub fn new(pool: PgPool, clock: Arc<dyn Clock>) -> Self {
        Self {
            org_repo: PgOrganizationRepository::new(pool.clone()),
            project_repo: PgProjectRepository::new(pool.clone()),
            project_key_repo: PgProjectKeyRepository::new(pool.clone()),
            project_role_repo: PgProjectRoleRepository::new(),
            project_member_repo: PgProjectMemberRepository::new(),
            pool,
            clock,
        }
    }
}

#[async_trait]
impl UnitOfWork for PgUnitOfWork {
    fn organizations(&self) -> &dyn OrganizationRepository {
        &self.org_repo
    }

    fn projects(&self) -> &dyn ProjectRepository {
        &self.project_repo
    }

    fn project_keys(&self) -> &dyn ProjectKeyRepository {
        &self.project_key_repo
    }

    fn project_roles(&self) -> &dyn ProjectRoleRepository {
        &self.project_role_repo
    }

    fn project_members(&self) -> &dyn ProjectMemberRepository {
        &self.project_member_repo
    }

    async fn save_changes(&mut self) -> Result<(), ApplicationError> {
        let org_entries = self.org_repo.take_entries();
        let project_entries = self.project_repo.take_entries();
        let key_entries = self.project_key_repo.take_entries();
        let role_entries = self.project_role_repo.take_entries();
        let member_entries = self.project_member_repo.take_entries();

        if org_entries.is_empty() && project_entries.is_empty()
            && key_entries.is_empty() && role_entries.is_empty()
            && member_entries.is_empty()
        {
            return Ok(());
        }

        let now = self.clock.now();
        let mut tx = self.pool.begin().await.map_err(map_sqlx_error)?;

        for entry in &org_entries {
            match entry {
                OrgEntry::Added(org) => {
                    OrganizationPersistence::insert(&mut tx, org, now).await?;
                }
                OrgEntry::Modified { entity, snapshot } => {
                    OrganizationPersistence::update(&mut tx, entity, snapshot, now).await?;
                }
                OrgEntry::Deleted(id) => {
                    OrganizationPersistence::delete(&mut tx, id).await?;
                }
            }
        }

        for entry in &project_entries {
            match entry {
                ProjectEntry::Added(project) => {
                    ProjectPersistence::insert(&mut tx, project, now).await?;
                }
                ProjectEntry::Modified { entity, snapshot } => {
                    ProjectPersistence::update(&mut tx, entity, snapshot, now).await?;
                }
                ProjectEntry::Deleted(id) => {
                    ProjectPersistence::delete(&mut tx, id).await?;
                }
            }
        }

        for entry in &key_entries {
            match entry {
                ProjectKeyEntry::Added(key) => {
                    ProjectKeyPersistence::insert(&mut tx, key, now).await?;
                }
                ProjectKeyEntry::Modified { entity, snapshot } => {
                    ProjectKeyPersistence::update(&mut tx, entity, snapshot, now).await?;
                }
            }
        }

        for entry in &role_entries {
            ProjectRolePersistence::insert(&mut tx, &entry.0, now).await?;
        }

        for entry in &member_entries {
            ProjectMemberPersistence::insert(&mut tx, &entry.0, now).await?;
        }

        tx.commit().await.map_err(map_sqlx_error)?;

        Ok(())
    }
}

pub struct PgUnitOfWorkFactory {
    pool: PgPool,
    clock: Arc<dyn Clock>,
}

impl PgUnitOfWorkFactory {
    pub fn new(pool: PgPool, clock: Arc<dyn Clock>) -> Self {
        Self { pool, clock }
    }
}

#[async_trait]
impl UnitOfWorkFactory for PgUnitOfWorkFactory {
    async fn create(&self) -> Result<Box<dyn UnitOfWork>, ApplicationError> {
        Ok(Box::new(PgUnitOfWork::new(self.pool.clone(), self.clock.clone())))
    }
}
