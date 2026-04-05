use std::sync::Arc;

use async_trait::async_trait;
use sqlx::PgPool;

use meerkat_application::error::ApplicationError;
use meerkat_application::ports::clock::Clock;
use meerkat_application::ports::oidc_config_warning_store::{
    OidcConfigWarningReadModel, OidcConfigWarningStore,
};
use meerkat_domain::models::oidc_config::OidcConfigId;

use super::error::map_sqlx_error;

pub struct PgOidcConfigWarningStore {
    pool: PgPool,
    clock: Arc<dyn Clock>,
}

impl PgOidcConfigWarningStore {
    pub fn new(pool: PgPool, clock: Arc<dyn Clock>) -> Self {
        Self { pool, clock }
    }
}

#[derive(sqlx::FromRow)]
struct WarningRow {
    oidc_config_id: sqlx::types::Uuid,
    warning_key: String,
    message: String,
    context: Option<serde_json::Value>,
    first_seen: chrono::DateTime<chrono::Utc>,
    last_seen: chrono::DateTime<chrono::Utc>,
    occurrence_count: i64,
}

#[async_trait]
impl OidcConfigWarningStore for PgOidcConfigWarningStore {
    async fn upsert(
        &self,
        oidc_config_id: &OidcConfigId,
        warning_key: &str,
        message: &str,
        context: Option<&serde_json::Value>,
    ) -> Result<(), ApplicationError> {
        let now = self.clock.now();

        sqlx::query(
            "INSERT INTO oidc_config_warnings (oidc_config_id, warning_key, message, context, first_seen, last_seen) \
             VALUES ($1, $2, $3, $4, $5, $5) \
             ON CONFLICT (oidc_config_id, warning_key) \
             DO UPDATE SET last_seen = $5, \
                           occurrence_count = oidc_config_warnings.occurrence_count + 1, \
                           message = EXCLUDED.message, \
                           context = EXCLUDED.context",
        )
        .bind(oidc_config_id.as_uuid())
        .bind(warning_key)
        .bind(message)
        .bind(context)
        .bind(now)
        .execute(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        Ok(())
    }

    async fn list_by_config(
        &self,
        oidc_config_id: &OidcConfigId,
    ) -> Result<Vec<OidcConfigWarningReadModel>, ApplicationError> {
        let rows = sqlx::query_as::<_, WarningRow>(
            "SELECT oidc_config_id, warning_key, message, context, first_seen, last_seen, occurrence_count \
             FROM oidc_config_warnings \
             WHERE oidc_config_id = $1 \
             ORDER BY last_seen DESC",
        )
        .bind(oidc_config_id.as_uuid())
        .fetch_all(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        Ok(rows
            .into_iter()
            .map(|r| OidcConfigWarningReadModel {
                oidc_config_id: OidcConfigId::from_uuid(r.oidc_config_id),
                warning_key: r.warning_key,
                message: r.message,
                context: r.context,
                first_seen: r.first_seen,
                last_seen: r.last_seen,
                occurrence_count: r.occurrence_count,
            })
            .collect())
    }

    async fn dismiss(
        &self,
        oidc_config_id: &OidcConfigId,
        warning_key: &str,
    ) -> Result<(), ApplicationError> {
        sqlx::query(
            "DELETE FROM oidc_config_warnings \
             WHERE oidc_config_id = $1 AND warning_key = $2",
        )
        .bind(oidc_config_id.as_uuid())
        .bind(warning_key)
        .execute(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        Ok(())
    }
}
