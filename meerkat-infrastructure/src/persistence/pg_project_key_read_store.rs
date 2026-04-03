use async_trait::async_trait;
use sqlx::PgPool;

use meerkat_application::error::ApplicationError;
use meerkat_application::ports::project_key_read_store::{ProjectKeyReadModel, ProjectKeyReadStore};
use meerkat_application::ports::project_read_store::PagedResult;
use meerkat_application::search::SearchFilter;
use meerkat_domain::models::project::ProjectId;
use meerkat_domain::models::project_key::{ProjectKeyId, ProjectKeyStatus};

use super::error::map_sqlx_error;

pub struct PgProjectKeyReadStore {
    pool: PgPool,
}

impl PgProjectKeyReadStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct ProjectKeyRow {
    id: sqlx::types::Uuid,
    project_id: sqlx::types::Uuid,
    key_token: String,
    label: String,
    status: String,
    created_at: chrono::DateTime<chrono::Utc>,
    total: i64,
}

#[async_trait]
impl ProjectKeyReadStore for PgProjectKeyReadStore {
    async fn list_by_project(
        &self,
        project_id: &ProjectId,
        search: Option<&SearchFilter>,
        limit: i64,
        offset: i64,
    ) -> Result<PagedResult<ProjectKeyReadModel>, ApplicationError> {
        let pattern = search.map(|s| s.contains_pattern());

        let rows = sqlx::query_as::<_, ProjectKeyRow>(
            "SELECT id, project_id, key_token, label, status, created_at, \
                    COUNT(*) OVER() AS total \
             FROM project_keys \
             WHERE project_id = $1 \
               AND ($4::text IS NULL OR label ILIKE $4) \
             ORDER BY created_at DESC \
             LIMIT $2 OFFSET $3",
        )
        .bind(project_id.as_uuid())
        .bind(limit)
        .bind(offset)
        .bind(pattern.as_deref())
        .fetch_all(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        let total = rows.first().map(|r| r.total).unwrap_or(0);

        let items = rows
            .into_iter()
            .map(|row| ProjectKeyReadModel {
                id: ProjectKeyId::from_uuid(row.id),
                project_id: ProjectId::from_uuid(row.project_id),
                key_token: row.key_token,
                label: row.label,
                status: row.status.parse::<ProjectKeyStatus>().expect("invalid status in database"),
                created_at: row.created_at,
            })
            .collect();

        Ok(PagedResult { items, total })
    }
}
