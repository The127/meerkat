use async_trait::async_trait;
use sqlx::PgPool;

use meerkat_application::error::ApplicationError;
use meerkat_application::ports::issue_read_store::{IssueReadModel, IssueReadStore};
use meerkat_application::ports::project_read_store::PagedResult;
use meerkat_application::search::SearchFilter;
use meerkat_domain::models::issue::IssueId;
use meerkat_domain::models::project::ProjectId;

use super::error::map_sqlx_error;

pub struct PgIssueReadStore {
    pool: PgPool,
}

impl PgIssueReadStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct IssueRow {
    id: sqlx::types::Uuid,
    project_id: sqlx::types::Uuid,
    title: String,
    fingerprint_hash: String,
    status: String,
    level: String,
    event_count: i64,
    first_seen: chrono::DateTime<chrono::Utc>,
    last_seen: chrono::DateTime<chrono::Utc>,
    total: i64,
}

#[async_trait]
impl IssueReadStore for PgIssueReadStore {
    async fn list_by_project(
        &self,
        project_id: &ProjectId,
        status: Option<&str>,
        search: Option<&SearchFilter>,
        limit: i64,
        offset: i64,
    ) -> Result<PagedResult<IssueReadModel>, ApplicationError> {
        let pattern = search.map(|s| s.contains_pattern());

        let rows = sqlx::query_as::<_, IssueRow>(
            "SELECT id, project_id, title, fingerprint_hash, status, level, event_count, \
                    first_seen, last_seen, \
                    COUNT(*) OVER() AS total \
             FROM issues \
             WHERE project_id = $1 \
               AND ($4::text IS NULL OR status = $4) \
               AND ($5::text IS NULL OR title ILIKE $5) \
             ORDER BY last_seen DESC \
             LIMIT $2 OFFSET $3",
        )
        .bind(project_id.as_uuid())
        .bind(limit)
        .bind(offset)
        .bind(status)
        .bind(pattern.as_deref())
        .fetch_all(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        let total = rows.first().map(|r| r.total).unwrap_or(0);

        let items = rows
            .into_iter()
            .map(|row| IssueReadModel {
                id: IssueId::from_uuid(row.id),
                project_id: ProjectId::from_uuid(row.project_id),
                title: row.title,
                fingerprint_hash: row.fingerprint_hash,
                status: row.status,
                level: row.level,
                event_count: row.event_count,
                first_seen: row.first_seen,
                last_seen: row.last_seen,
            })
            .collect();

        Ok(PagedResult { items, total })
    }
}
