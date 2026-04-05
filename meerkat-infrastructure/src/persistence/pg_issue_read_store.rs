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
    issue_number: i64,
    title: String,
    fingerprint_hash: String,
    status: String,
    level: String,
    event_count: i64,
    first_seen: chrono::DateTime<chrono::Utc>,
    last_seen: chrono::DateTime<chrono::Utc>,
    total: i64,
}

fn to_read_model(row: &IssueRow) -> IssueReadModel {
    IssueReadModel {
        id: IssueId::from_uuid(row.id),
        project_id: ProjectId::from_uuid(row.project_id),
        issue_number: row.issue_number,
        title: row.title.clone(),
        fingerprint_hash: row.fingerprint_hash.clone(),
        status: row.status.clone(),
        level: row.level.clone(),
        event_count: row.event_count,
        first_seen: row.first_seen,
        last_seen: row.last_seen,
    }
}

#[async_trait]
impl IssueReadStore for PgIssueReadStore {
    async fn find_by_number(
        &self,
        project_id: &ProjectId,
        issue_number: i64,
    ) -> Result<Option<IssueReadModel>, ApplicationError> {
        let row = sqlx::query_as::<_, IssueRow>(
            "SELECT id, project_id, issue_number, title, fingerprint_hash, status, level, event_count, \
                    first_seen, last_seen, 0::bigint AS total \
             FROM issues \
             WHERE project_id = $1 AND issue_number = $2",
        )
        .bind(project_id.as_uuid())
        .bind(issue_number)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        Ok(row.as_ref().map(to_read_model))
    }

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
            "SELECT id, project_id, issue_number, title, fingerprint_hash, status, level, event_count, \
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
        let items = rows.iter().map(to_read_model).collect();

        Ok(PagedResult { items, total })
    }
}
