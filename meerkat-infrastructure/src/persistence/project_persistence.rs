use chrono::{DateTime, Utc};

use meerkat_application::error::ApplicationError;
use meerkat_domain::models::project::{Project, ProjectId};

use super::error::map_sqlx_error;

pub(crate) struct ProjectPersistence;

impl ProjectPersistence {
    pub async fn insert(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        project: &Project,
        now: DateTime<Utc>,
    ) -> Result<(), ApplicationError> {
        sqlx::query(
            "INSERT INTO projects (id, organization_id, name, slug, created_at, updated_at, version) \
             VALUES ($1, $2, $3, $4, $5, $6, $7)",
        )
        .bind(project.id().as_uuid())
        .bind(project.organization_id().as_uuid())
        .bind(project.name())
        .bind(project.slug().as_str())
        .bind(now)
        .bind(now)
        .bind(project.version().as_u64() as i64)
        .execute(&mut **tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(())
    }

    pub async fn update(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        project: &Project,
        snapshot: &Project,
        now: DateTime<Utc>,
    ) -> Result<(), ApplicationError> {
        let changed = project.name() != snapshot.name()
            || project.slug() != snapshot.slug();

        if !changed {
            return Ok(());
        }

        let new_version = snapshot.version().increment();

        let result = sqlx::query(
            "UPDATE projects SET name = $1, slug = $2, updated_at = $3, version = $4 \
             WHERE id = $5 AND version = $6",
        )
        .bind(project.name())
        .bind(project.slug().as_str())
        .bind(now)
        .bind(new_version.as_u64() as i64)
        .bind(project.id().as_uuid())
        .bind(snapshot.version().as_u64() as i64)
        .execute(&mut **tx)
        .await
        .map_err(map_sqlx_error)?;

        if result.rows_affected() == 0 {
            return Err(ApplicationError::Conflict);
        }

        Ok(())
    }

    pub async fn delete(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        id: &ProjectId,
    ) -> Result<(), ApplicationError> {
        sqlx::query("DELETE FROM projects WHERE id = $1")
            .bind(id.as_uuid())
            .execute(&mut **tx)
            .await
            .map_err(map_sqlx_error)?;

        Ok(())
    }
}
