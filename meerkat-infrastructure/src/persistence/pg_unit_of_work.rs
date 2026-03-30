use async_trait::async_trait;
use sqlx::PgPool;

use meerkat_application::error::ApplicationError;
use meerkat_application::ports::organization_store::WriteOrganizationStore;
use meerkat_application::ports::project_store::WriteProjectStore;
use meerkat_application::ports::unit_of_work::{UnitOfWork, UnitOfWorkFactory};
use meerkat_domain::models::organization::OrganizationChange;
use meerkat_domain::models::project::ProjectChange;

use crate::persistence::pg_organization_store::PgWriteOrganizationStore;
use crate::persistence::pg_project_store::PgWriteProjectStore;

pub struct PgUnitOfWork {
    pool: PgPool,
    org_store: PgWriteOrganizationStore,
    project_store: PgWriteProjectStore,
}

impl PgUnitOfWork {
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool,
            org_store: PgWriteOrganizationStore::new(),
            project_store: PgWriteProjectStore::new(),
        }
    }
}

#[async_trait]
impl UnitOfWork for PgUnitOfWork {
    fn organizations(&self) -> &dyn WriteOrganizationStore {
        &self.org_store
    }

    fn projects(&self) -> &dyn WriteProjectStore {
        &self.project_store
    }

    async fn save_changes(&mut self) -> Result<(), ApplicationError> {
        let mut orgs = self.org_store.take_buffered();
        let mut projects = self.project_store.take_buffered();

        if orgs.is_empty() && projects.is_empty() {
            return Ok(());
        }

        let mut tx = self.pool.begin().await.map_err(map_sqlx_error)?;

        for org in &mut orgs {
            let changes = org.pull_changes();

            if changes.is_empty() {
                continue;
            }

            let is_new = changes.iter().any(|c| matches!(c, OrganizationChange::Created { .. }));

            if is_new {
                sqlx::query(
                    "INSERT INTO organizations (id, name, slug, created_at, updated_at, version) \
                     VALUES ($1, $2, $3, $4, $5, $6)"
                )
                .bind(org.id().as_uuid())
                .bind(org.name())
                .bind(org.slug().as_str())
                .bind(org.created_at())
                .bind(org.updated_at())
                .bind(org.version().as_u64() as i64)
                .execute(&mut *tx)
                .await
                .map_err(map_sqlx_error)?;
            } else {
                let new_version = org.version().increment();

                let result = sqlx::query(
                    "UPDATE organizations SET name = $1, slug = $2, updated_at = $3, version = $4 \
                     WHERE id = $5 AND version = $6"
                )
                .bind(org.name())
                .bind(org.slug().as_str())
                .bind(org.updated_at())
                .bind(new_version.as_u64() as i64)
                .bind(org.id().as_uuid())
                .bind(org.version().as_u64() as i64)
                .execute(&mut *tx)
                .await
                .map_err(map_sqlx_error)?;

                if result.rows_affected() == 0 {
                    return Err(ApplicationError::Conflict);
                }
            }
        }

        for project in &mut projects {
            let changes = project.pull_changes();

            if changes.is_empty() {
                continue;
            }

            let is_new = changes.iter().any(|c| matches!(c, ProjectChange::Created { .. }));

            if is_new {
                sqlx::query(
                    "INSERT INTO projects (id, organization_id, name, slug, created_at, updated_at, version) \
                     VALUES ($1, $2, $3, $4, $5, $6, $7)"
                )
                .bind(project.id().as_uuid())
                .bind(project.organization_id().as_uuid())
                .bind(project.name())
                .bind(project.slug().as_str())
                .bind(project.created_at())
                .bind(project.updated_at())
                .bind(project.version().as_u64() as i64)
                .execute(&mut *tx)
                .await
                .map_err(map_sqlx_error)?;
            } else {
                let new_version = project.version().increment();

                let result = sqlx::query(
                    "UPDATE projects SET name = $1, slug = $2, updated_at = $3, version = $4 \
                     WHERE id = $5 AND version = $6"
                )
                .bind(project.name())
                .bind(project.slug().as_str())
                .bind(project.updated_at())
                .bind(new_version.as_u64() as i64)
                .bind(project.id().as_uuid())
                .bind(project.version().as_u64() as i64)
                .execute(&mut *tx)
                .await
                .map_err(map_sqlx_error)?;

                if result.rows_affected() == 0 {
                    return Err(ApplicationError::Conflict);
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

fn map_sqlx_error(err: sqlx::Error) -> ApplicationError {
    if let sqlx::Error::Database(ref db_err) = err
        && db_err.code().as_deref() == Some("23505")
    {
        return ApplicationError::Conflict;
    }
    ApplicationError::Internal(err.to_string())
}
