use chrono::{DateTime, Utc};

use meerkat_application::error::ApplicationError;
use meerkat_domain::models::event::Event;

use super::error::map_sqlx_error;

pub(crate) struct EventPersistence;

impl EventPersistence {
    pub async fn insert(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        event: &Event,
        now: DateTime<Utc>,
    ) -> Result<(), ApplicationError> {
        let tags = serde_json::to_value(event.tags())
            .map_err(|e| ApplicationError::Internal(e.to_string()))?;

        sqlx::query(
            "INSERT INTO events (id, project_id, issue_id, fingerprint_hash, message, level, platform, \
             timestamp, server_name, environment, release, exception_type, exception_value, \
             tags, extra, created_at, updated_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17)",
        )
        .bind(event.id().as_uuid())
        .bind(event.project_id().as_uuid())
        .bind(event.issue_id().as_uuid())
        .bind(event.fingerprint_hash().as_str())
        .bind(event.message())
        .bind(event.level().as_ref())
        .bind(event.platform())
        .bind(event.timestamp())
        .bind(event.server_name())
        .bind(event.environment())
        .bind(event.release())
        .bind(event.exception_type())
        .bind(event.exception_value())
        .bind(&tags)
        .bind(event.extra())
        .bind(now)
        .bind(now)
        .execute(&mut **tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(())
    }
}
