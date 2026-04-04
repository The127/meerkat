use async_trait::async_trait;
use sqlx::PgPool;

use meerkat_application::error::ApplicationError;
use meerkat_application::ports::event_repository::EventRepository;
use meerkat_domain::models::event::Event;

use super::error::map_sqlx_error;

pub struct PgEventRepository {
    pool: PgPool,
}

impl PgEventRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl EventRepository for PgEventRepository {
    async fn add(&self, event: &Event) -> Result<(), ApplicationError> {
        let tags = serde_json::to_value(event.tags()).map_err(|e| ApplicationError::Internal(e.to_string()))?;
        let now = chrono::Utc::now();

        sqlx::query(
            "INSERT INTO events (id, project_id, issue_id, fingerprint_hash, message, level, platform, \
             timestamp, server_name, environment, release, exception_type, exception_value, \
             tags, extra, created_at, updated_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17)",
        )
        .bind(event.id().as_uuid())
        .bind(event.project_id().as_uuid())
        .bind(event.issue_id().as_uuid())
        .bind(event.fingerprint_hash())
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
        .execute(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        Ok(())
    }
}
