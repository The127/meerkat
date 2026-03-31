use async_trait::async_trait;
use sqlx::PgPool;

use meerkat_application::error::ApplicationError;
use meerkat_application::ports::organization_repository::OrganizationRepository;
use meerkat_application::ports::project_repository::ProjectRepository;
use meerkat_application::ports::unit_of_work::{UnitOfWork, UnitOfWorkFactory};

use crate::persistence::error::map_sqlx_error;
use crate::persistence::organization_persistence::OrganizationPersistence;
use crate::persistence::pg_organization_repository::{OrgEntry, PgOrganizationRepository};
use crate::persistence::pg_project_repository::{PgProjectRepository, ProjectEntry};
use crate::persistence::project_persistence::ProjectPersistence;

pub struct PgUnitOfWork {
    pool: PgPool,
    org_repo: PgOrganizationRepository,
    project_repo: PgProjectRepository,
}

impl PgUnitOfWork {
    pub fn new(pool: PgPool) -> Self {
        Self {
            org_repo: PgOrganizationRepository::new(pool.clone()),
            project_repo: PgProjectRepository::new(pool.clone()),
            pool,
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

    async fn save_changes(&mut self) -> Result<(), ApplicationError> {
        let org_entries = self.org_repo.take_entries();
        let project_entries = self.project_repo.take_entries();

        if org_entries.is_empty() && project_entries.is_empty() {
            return Ok(());
        }

        let mut tx = self.pool.begin().await.map_err(map_sqlx_error)?;

        for entry in &org_entries {
            match entry {
                OrgEntry::Added(org) => {
                    OrganizationPersistence::insert(&mut tx, org).await?;
                }
                OrgEntry::Modified { entity, snapshot } => {
                    OrganizationPersistence::update(&mut tx, entity, snapshot).await?;
                }
                OrgEntry::Deleted(id) => {
                    OrganizationPersistence::delete(&mut tx, id).await?;
                }
            }
        }

        for entry in &project_entries {
            match entry {
                ProjectEntry::Added(project) => {
                    ProjectPersistence::insert(&mut tx, project).await?;
                }
                ProjectEntry::Modified { entity, snapshot } => {
                    ProjectPersistence::update(&mut tx, entity, snapshot).await?;
                }
                ProjectEntry::Deleted(id) => {
                    ProjectPersistence::delete(&mut tx, id).await?;
                }
            }
        }

        tx.commit().await.map_err(map_sqlx_error)?;

        Ok(())
    }
}

pub struct PgUnitOfWorkFactory {
    pool: PgPool,
}

impl PgUnitOfWorkFactory {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl UnitOfWorkFactory for PgUnitOfWorkFactory {
    async fn create(&self) -> Result<Box<dyn UnitOfWork>, ApplicationError> {
        Ok(Box::new(PgUnitOfWork::new(self.pool.clone())))
    }
}

