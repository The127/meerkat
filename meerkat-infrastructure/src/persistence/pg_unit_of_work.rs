use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;

use meerkat_application::error::ApplicationError;
use meerkat_application::ports::event_repository::EventRepository;
use meerkat_application::ports::issue_repository::IssueRepository;
use meerkat_application::ports::organization_repository::OrganizationRepository;
use meerkat_application::ports::project_key_repository::ProjectKeyRepository;
use meerkat_application::ports::project_member_repository::ProjectMemberRepository;
use meerkat_application::ports::project_repository::ProjectRepository;
use meerkat_application::ports::project_role_repository::ProjectRoleRepository;
use meerkat_application::ports::unit_of_work::{UnitOfWork, UnitOfWorkFactory};
use meerkat_application::ports::clock::Clock;

use crate::persistence::error::map_sqlx_error;
use crate::persistence::event_persistence::EventPersistence;
use crate::persistence::issue_persistence::IssuePersistence;
use crate::persistence::organization_persistence::OrganizationPersistence;
use crate::persistence::pg_event_repository::{EventEntry, PgEventRepository};
use crate::persistence::pg_issue_repository::{IssueEntry, PgIssueRepository};
use crate::persistence::pg_organization_repository::{OrgEntry, PgOrganizationRepository};
use crate::persistence::pg_project_key_repository::{PgProjectKeyRepository, ProjectKeyEntry};
use crate::persistence::pg_project_member_repository::{PgProjectMemberRepository, ProjectMemberEntry};
use crate::persistence::pg_project_repository::{PgProjectRepository, ProjectEntry};
use crate::persistence::pg_project_role_repository::{PgProjectRoleRepository, ProjectRoleEntry};
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
    event_repo: PgEventRepository,
    issue_repo: PgIssueRepository,
}

impl PgUnitOfWork {
    pub fn new(pool: PgPool, clock: Arc<dyn Clock>) -> Self {
        Self {
            org_repo: PgOrganizationRepository::new(pool.clone()),
            project_repo: PgProjectRepository::new(pool.clone()),
            project_key_repo: PgProjectKeyRepository::new(pool.clone()),
            project_role_repo: PgProjectRoleRepository::new(),
            project_member_repo: PgProjectMemberRepository::new(),
            event_repo: PgEventRepository::new(),
            issue_repo: PgIssueRepository::new(pool.clone()),
            pool,
            clock,
        }
    }

    async fn save_organization_entries(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        entries: &[OrgEntry],
        now: DateTime<Utc>,
    ) -> Result<(), ApplicationError> {
        for entry in entries {
            match entry {
                OrgEntry::Added(org) => {
                    OrganizationPersistence::insert(tx, org, now).await?;
                }
                OrgEntry::Modified { entity, snapshot } => {
                    OrganizationPersistence::update(tx, entity, snapshot, now).await?;
                }
                OrgEntry::Deleted(id) => {
                    OrganizationPersistence::delete(tx, id).await?;
                }
            }
        }
        Ok(())
    }

    async fn save_project_entries(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        entries: &[ProjectEntry],
        now: DateTime<Utc>,
    ) -> Result<(), ApplicationError> {
        for entry in entries {
            match entry {
                ProjectEntry::Added(project) => {
                    ProjectPersistence::insert(tx, project, now).await?;
                }
                ProjectEntry::Modified { entity, snapshot } => {
                    ProjectPersistence::update(tx, entity, snapshot, now).await?;
                }
                ProjectEntry::Deleted(id) => {
                    ProjectPersistence::delete(tx, id).await?;
                }
            }
        }
        Ok(())
    }

    async fn save_project_key_entries(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        entries: &[ProjectKeyEntry],
        now: DateTime<Utc>,
    ) -> Result<(), ApplicationError> {
        for entry in entries {
            match entry {
                ProjectKeyEntry::Added(key) => {
                    ProjectKeyPersistence::insert(tx, key, now).await?;
                }
                ProjectKeyEntry::Modified { entity, snapshot } => {
                    ProjectKeyPersistence::update(tx, entity, snapshot, now).await?;
                }
            }
        }
        Ok(())
    }

    async fn save_project_role_entries(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        entries: &[ProjectRoleEntry],
        now: DateTime<Utc>,
    ) -> Result<(), ApplicationError> {
        for entry in entries {
            ProjectRolePersistence::insert(tx, &entry.0, now).await?;
        }
        Ok(())
    }

    async fn save_project_member_entries(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        entries: &[ProjectMemberEntry],
        now: DateTime<Utc>,
    ) -> Result<(), ApplicationError> {
        for entry in entries {
            ProjectMemberPersistence::insert(tx, &entry.0, now).await?;
        }
        Ok(())
    }

    async fn save_event_entries(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        entries: &[EventEntry],
        now: DateTime<Utc>,
    ) -> Result<(), ApplicationError> {
        for entry in entries {
            EventPersistence::insert(tx, &entry.0, now).await?;
        }
        Ok(())
    }

    async fn save_issue_entries(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        entries: &[IssueEntry],
        now: DateTime<Utc>,
    ) -> Result<(), ApplicationError> {
        for entry in entries {
            match entry {
                IssueEntry::Added(issue) => {
                    IssuePersistence::insert(tx, issue, now).await?;
                }
                IssueEntry::Modified { entity, snapshot } => {
                    IssuePersistence::update(tx, entity, snapshot, now).await?;
                }
            }
        }
        Ok(())
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

    fn events(&self) -> &dyn EventRepository {
        &self.event_repo
    }

    fn issues(&self) -> &dyn IssueRepository {
        &self.issue_repo
    }

    async fn save_changes(&mut self) -> Result<(), ApplicationError> {
        let org_entries = self.org_repo.take_entries();
        let project_entries = self.project_repo.take_entries();
        let key_entries = self.project_key_repo.take_entries();
        let role_entries = self.project_role_repo.take_entries();
        let member_entries = self.project_member_repo.take_entries();
        let event_entries = self.event_repo.take_entries();
        let issue_entries = self.issue_repo.take_entries();

        if org_entries.is_empty() && project_entries.is_empty()
            && key_entries.is_empty() && role_entries.is_empty()
            && member_entries.is_empty() && event_entries.is_empty()
            && issue_entries.is_empty()
        {
            return Ok(());
        }

        let now = self.clock.now();
        let mut tx = self.pool.begin().await.map_err(map_sqlx_error)?;

        Self::save_organization_entries(&mut tx, &org_entries, now).await?;
        Self::save_project_entries(&mut tx, &project_entries, now).await?;
        Self::save_project_key_entries(&mut tx, &key_entries, now).await?;
        Self::save_project_role_entries(&mut tx, &role_entries, now).await?;
        Self::save_project_member_entries(&mut tx, &member_entries, now).await?;
        Self::save_issue_entries(&mut tx, &issue_entries, now).await?;
        Self::save_event_entries(&mut tx, &event_entries, now).await?;

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
