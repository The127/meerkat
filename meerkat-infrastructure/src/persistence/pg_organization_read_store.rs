use async_trait::async_trait;
use sqlx::PgPool;

use meerkat_application::error::ApplicationError;
use meerkat_application::ports::organization_read_store::{OrganizationReadModel, OrganizationReadStore};
use meerkat_domain::models::organization::{OrganizationId, OrganizationSlug};

pub struct PgOrganizationReadStore {
    pool: PgPool,
}

impl PgOrganizationReadStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct OrganizationRow {
    id: sqlx::types::Uuid,
    slug: String,
    name: String,
}

impl From<OrganizationRow> for OrganizationReadModel {
    fn from(row: OrganizationRow) -> Self {
        Self {
            id: OrganizationId::from_uuid(row.id),
            slug: OrganizationSlug::new(row.slug).expect("invalid slug in database"),
            name: row.name,
        }
    }
}

#[async_trait]
impl OrganizationReadStore for PgOrganizationReadStore {
    async fn any_exists(&self) -> Result<bool, ApplicationError> {
        let exists = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(SELECT 1 FROM organizations)"
        )
        .fetch_one(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        Ok(exists)
    }

    async fn find_by_slug(
        &self,
        slug: &OrganizationSlug,
    ) -> Result<Option<OrganizationReadModel>, ApplicationError> {
        let row = sqlx::query_as::<_, OrganizationRow>(
            "SELECT id, slug, name FROM organizations WHERE slug = $1"
        )
        .bind(slug.as_str())
        .fetch_optional(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        Ok(row.map(Into::into))
    }
}

fn map_sqlx_error(err: sqlx::Error) -> ApplicationError {
    ApplicationError::Internal(err.to_string())
}
