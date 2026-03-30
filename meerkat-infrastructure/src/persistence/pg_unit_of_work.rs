use async_trait::async_trait;
use sqlx::PgPool;

use meerkat_application::error::ApplicationError;
use meerkat_application::ports::organization_store::WriteOrganizationStore;
use meerkat_application::ports::unit_of_work::{UnitOfWork, UnitOfWorkFactory};
use meerkat_domain::models::organization::OrganizationChange;

use crate::persistence::pg_organization_store::PgWriteOrganizationStore;

pub struct PgUnitOfWork {
    pool: PgPool,
    org_store: PgWriteOrganizationStore,
}

impl PgUnitOfWork {
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool,
            org_store: PgWriteOrganizationStore::new(),
        }
    }
}

#[async_trait]
impl UnitOfWork for PgUnitOfWork {
    fn organizations(&self) -> &dyn WriteOrganizationStore {
        &self.org_store
    }

    async fn save_changes(&mut self) -> Result<(), ApplicationError> {
        let mut orgs = self.org_store.take_buffered();

        if orgs.is_empty() {
            return Ok(());
        }

        let mut tx = self.pool.begin().await.map_err(map_sqlx_error)?;

        for org in &mut orgs {
            let changes = org.pull_changes();
            for change in changes {
                match change {
                    OrganizationChange::Created { id, name, slug } => {
                        sqlx::query(
                            "INSERT INTO organizations (id, name, slug, created_at, updated_at, version) \
                             VALUES ($1, $2, $3, $4, $5, $6)"
                        )
                        .bind(id.as_uuid())
                        .bind(&name)
                        .bind(slug.as_str())
                        .bind(org.created_at())
                        .bind(org.updated_at())
                        .bind(org.version().as_u64() as i64)
                        .execute(&mut *tx)
                        .await
                        .map_err(map_sqlx_error)?;
                    }
                    OrganizationChange::NameUpdated { id, old_name: _, new_name } => {
                        let previous_version = (org.version().as_u64() - 1) as i64;

                        let result = sqlx::query(
                            "UPDATE organizations SET name = $1, updated_at = $2, version = $3 \
                             WHERE id = $4 AND version = $5"
                        )
                        .bind(&new_name)
                        .bind(org.updated_at())
                        .bind(org.version().as_u64() as i64)
                        .bind(id.as_uuid())
                        .bind(previous_version)
                        .execute(&mut *tx)
                        .await
                        .map_err(map_sqlx_error)?;

                        if result.rows_affected() == 0 {
                            return Err(ApplicationError::Conflict);
                        }
                    }
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
