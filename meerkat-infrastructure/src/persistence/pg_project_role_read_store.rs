use async_trait::async_trait;
use sqlx::PgPool;

use meerkat_application::error::ApplicationError;
use meerkat_application::ports::project_role_read_store::{ProjectRoleReadModel, ProjectRoleReadStore};
use meerkat_domain::models::permission::ProjectPermission;
use meerkat_domain::models::project::ProjectId;
use meerkat_domain::models::project_role::{ProjectRoleId, ProjectRoleSlug};

use super::error::map_sqlx_error;

pub struct PgProjectRoleReadStore {
    pool: PgPool,
}

impl PgProjectRoleReadStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct RoleRow {
    id: sqlx::types::Uuid,
    name: String,
    slug: String,
    is_default: bool,
    permissions: Vec<String>,
}

impl From<RoleRow> for ProjectRoleReadModel {
    fn from(row: RoleRow) -> Self {
        Self {
            id: ProjectRoleId::from_uuid(row.id),
            name: row.name,
            slug: ProjectRoleSlug::new(row.slug).expect("invalid slug in database"),
            permissions: row.permissions.into_iter().filter_map(|s| s.parse::<ProjectPermission>().ok()).collect(),
            is_default: row.is_default,
        }
    }
}

#[async_trait]
impl ProjectRoleReadStore for PgProjectRoleReadStore {
    async fn list_by_project(
        &self,
        project_id: &ProjectId,
    ) -> Result<Vec<ProjectRoleReadModel>, ApplicationError> {
        let roles = sqlx::query_as::<_, RoleRow>(
            "SELECT r.id, r.name, r.slug, r.is_default, \
                    COALESCE(array_agg(p.permission ORDER BY p.permission) \
                             FILTER (WHERE p.permission IS NOT NULL), '{}') AS permissions \
             FROM project_roles r \
             LEFT JOIN project_role_permissions p ON p.role_id = r.id \
             WHERE r.project_id = $1 \
             GROUP BY r.id, r.name, r.slug, r.is_default \
             ORDER BY r.is_default DESC, r.name",
        )
        .bind(project_id.as_uuid())
        .fetch_all(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        Ok(roles.into_iter().map(ProjectRoleReadModel::from).collect())
    }

    async fn find_by_id(
        &self,
        id: &ProjectRoleId,
    ) -> Result<Option<ProjectRoleReadModel>, ApplicationError> {
        let row = sqlx::query_as::<_, RoleRow>(
            "SELECT r.id, r.name, r.slug, r.is_default, \
                    COALESCE(array_agg(p.permission ORDER BY p.permission) \
                             FILTER (WHERE p.permission IS NOT NULL), '{}') AS permissions \
             FROM project_roles r \
             LEFT JOIN project_role_permissions p ON p.role_id = r.id \
             WHERE r.id = $1 \
             GROUP BY r.id, r.name, r.slug, r.is_default",
        )
        .bind(id.as_uuid())
        .fetch_optional(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        Ok(row.map(ProjectRoleReadModel::from))
    }
}
