use chrono::{DateTime, Utc};

use meerkat_application::error::ApplicationError;
use meerkat_domain::models::project_key::ProjectKey;

use super::error::map_sqlx_error;

pub(crate) struct ProjectKeyPersistence;

impl ProjectKeyPersistence {
    pub async fn insert(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        key: &ProjectKey,
        now: DateTime<Utc>,
    ) -> Result<(), ApplicationError> {
        sqlx::query(
            "INSERT INTO project_keys (id, project_id, key_token, label, status, created_at, updated_at, version) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
        )
        .bind(key.id().as_uuid())
        .bind(key.project_id().as_uuid())
        .bind(key.key_token().as_str())
        .bind(key.label())
        .bind(key.status().as_ref())
        .bind(now)
        .bind(now)
        .bind(key.version().as_u64() as i64)
        .execute(&mut **tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(())
    }

    pub async fn update(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        key: &ProjectKey,
        snapshot: &ProjectKey,
        now: DateTime<Utc>,
    ) -> Result<(), ApplicationError> {
        let changed = key.status() != snapshot.status()
            || key.label() != snapshot.label();

        if !changed {
            return Ok(());
        }

        let new_version = snapshot.version().increment();

        let result = sqlx::query(
            "UPDATE project_keys SET status = $1, label = $2, updated_at = $3, version = $4 \
             WHERE id = $5 AND version = $6",
        )
        .bind(key.status().as_ref())
        .bind(key.label())
        .bind(now)
        .bind(new_version.as_u64() as i64)
        .bind(key.id().as_uuid())
        .bind(snapshot.version().as_u64() as i64)
        .execute(&mut **tx)
        .await
        .map_err(map_sqlx_error)?;

        if result.rows_affected() == 0 {
            return Err(ApplicationError::Conflict);
        }

        Ok(())
    }
}
