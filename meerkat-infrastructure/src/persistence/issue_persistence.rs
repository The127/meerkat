use chrono::{DateTime, Utc};

use meerkat_application::error::ApplicationError;
use meerkat_domain::models::issue::Issue;

use super::error::map_sqlx_error;

pub(crate) struct IssuePersistence;

impl IssuePersistence {
    pub async fn insert(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        issue: &Issue,
        now: DateTime<Utc>,
    ) -> Result<(), ApplicationError> {
        sqlx::query(
            "INSERT INTO issues (id, project_id, title, fingerprint_hash, status, level, \
             event_count, first_seen, last_seen, version, created_at, updated_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)",
        )
        .bind(issue.id().as_uuid())
        .bind(issue.project_id().as_uuid())
        .bind(issue.title())
        .bind(issue.fingerprint_hash().as_str())
        .bind(issue.status().as_ref())
        .bind(issue.level().as_ref())
        .bind(issue.event_count() as i64)
        .bind(issue.first_seen())
        .bind(issue.last_seen())
        .bind(issue.version().as_u64() as i64)
        .bind(now)
        .bind(now)
        .execute(&mut **tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(())
    }

    pub async fn update(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        issue: &Issue,
        snapshot: &Issue,
        now: DateTime<Utc>,
    ) -> Result<(), ApplicationError> {
        let new_version = snapshot.version().increment();

        let result = sqlx::query(
            "UPDATE issues SET title = $1, status = $2, level = $3, event_count = $4, \
             first_seen = $5, last_seen = $6, version = $7, updated_at = $8 \
             WHERE id = $9 AND version = $10",
        )
        .bind(issue.title())
        .bind(issue.status().as_ref())
        .bind(issue.level().as_ref())
        .bind(issue.event_count() as i64)
        .bind(issue.first_seen())
        .bind(issue.last_seen())
        .bind(new_version.as_u64() as i64)
        .bind(now)
        .bind(issue.id().as_uuid())
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
