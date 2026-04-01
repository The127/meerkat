use async_trait::async_trait;
use sqlx::PgPool;

use meerkat_application::error::ApplicationError;
use meerkat_application::ports::project_read_store::{PagedResult, ProjectReadModel, ProjectReadStore};
use meerkat_application::search::SearchFilter;
use meerkat_domain::models::organization::OrganizationId;
use meerkat_domain::models::project::{ProjectId, ProjectSlug};

use super::error::map_sqlx_error;

pub struct PgProjectReadStore {
    pool: PgPool,
}

impl PgProjectReadStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct ProjectRow {
    id: sqlx::types::Uuid,
    organization_id: sqlx::types::Uuid,
    name: String,
    slug: String,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
    total: i64,
}

#[async_trait]
impl ProjectReadStore for PgProjectReadStore {
    async fn list_by_org(
        &self,
        org_id: &OrganizationId,
        search: Option<&SearchFilter>,
        limit: i64,
        offset: i64,
    ) -> Result<PagedResult<ProjectReadModel>, ApplicationError> {
        let pattern = search.map(|s| s.contains_pattern());

        let rows = sqlx::query_as::<_, ProjectRow>(
            "SELECT id, organization_id, name, slug, created_at, updated_at, \
                    COUNT(*) OVER() AS total \
             FROM projects \
             WHERE organization_id = $1 \
               AND ($4::text IS NULL OR name ILIKE $4 OR slug ILIKE $4) \
             ORDER BY created_at DESC \
             LIMIT $2 OFFSET $3",
        )
        .bind(org_id.as_uuid())
        .bind(limit)
        .bind(offset)
        .bind(pattern.as_deref())
        .fetch_all(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        let total = rows.first().map(|r| r.total).unwrap_or(0);

        let items = rows
            .into_iter()
            .map(|row| ProjectReadModel {
                id: ProjectId::from_uuid(row.id),
                organization_id: OrganizationId::from_uuid(row.organization_id),
                name: row.name,
                slug: ProjectSlug::new(row.slug).expect("invalid slug in database"),
                created_at: row.created_at,
                updated_at: row.updated_at,
            })
            .collect();

        Ok(PagedResult { items, total })
    }
}
