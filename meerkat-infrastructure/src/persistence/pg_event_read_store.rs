use async_trait::async_trait;
use sqlx::PgPool;

use meerkat_application::error::ApplicationError;
use meerkat_application::ports::event_read_store::{EventReadModel, EventReadStore};
use meerkat_application::ports::project_read_store::PagedResult;
use meerkat_domain::models::event::EventId;
use meerkat_domain::models::issue::IssueId;
use meerkat_domain::models::project::ProjectId;

use super::error::map_sqlx_error;

pub struct PgEventReadStore {
    pool: PgPool,
}

impl PgEventReadStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct EventRow {
    id: sqlx::types::Uuid,
    project_id: sqlx::types::Uuid,
    issue_id: sqlx::types::Uuid,
    fingerprint_hash: String,
    message: String,
    level: String,
    platform: String,
    timestamp: chrono::DateTime<chrono::Utc>,
    server_name: Option<String>,
    environment: Option<String>,
    release: Option<String>,
    exception_type: Option<String>,
    exception_value: Option<String>,
    tags: serde_json::Value,
    extra: serde_json::Value,
    total: i64,
}

fn parse_tags(value: serde_json::Value) -> Vec<(String, String)> {
    match value {
        serde_json::Value::Array(arr) => arr
            .into_iter()
            .filter_map(|item| {
                let arr = item.as_array()?;
                if arr.len() == 2 {
                    Some((
                        arr[0].as_str()?.to_string(),
                        arr[1].as_str()?.to_string(),
                    ))
                } else {
                    None
                }
            })
            .collect(),
        _ => vec![],
    }
}

#[async_trait]
impl EventReadStore for PgEventReadStore {
    async fn list_by_issue(
        &self,
        issue_id: &IssueId,
        limit: i64,
        offset: i64,
    ) -> Result<PagedResult<EventReadModel>, ApplicationError> {
        let rows = sqlx::query_as::<_, EventRow>(
            "SELECT id, project_id, issue_id, fingerprint_hash, message, level, platform, \
                    timestamp, server_name, environment, release, exception_type, exception_value, \
                    tags, extra, \
                    COUNT(*) OVER() AS total \
             FROM events \
             WHERE issue_id = $1 \
             ORDER BY timestamp DESC \
             LIMIT $2 OFFSET $3",
        )
        .bind(issue_id.as_uuid())
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        let total = rows.first().map(|r| r.total).unwrap_or(0);

        let items = rows
            .into_iter()
            .map(|row| EventReadModel {
                id: EventId::from_uuid(row.id),
                project_id: ProjectId::from_uuid(row.project_id),
                issue_id: IssueId::from_uuid(row.issue_id),
                fingerprint_hash: row.fingerprint_hash,
                message: row.message,
                level: row.level,
                platform: row.platform,
                timestamp: row.timestamp,
                server_name: row.server_name,
                environment: row.environment,
                release: row.release,
                exception_type: row.exception_type,
                exception_value: row.exception_value,
                tags: parse_tags(row.tags),
                extra: row.extra,
            })
            .collect();

        Ok(PagedResult { items, total })
    }
}
