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
}

#[async_trait]
impl ProjectRoleReadStore for PgProjectRoleReadStore {
    async fn list_by_project(
        &self,
        project_id: &ProjectId,
    ) -> Result<Vec<ProjectRoleReadModel>, ApplicationError> {
        let roles = sqlx::query_as::<_, RoleRow>(
            "SELECT id, name, slug, is_default \
             FROM project_roles \
             WHERE project_id = $1 \
             ORDER BY is_default DESC, name",
        )
        .bind(project_id.as_uuid())
        .fetch_all(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        let mut result = Vec::with_capacity(roles.len());
        for role in roles {
            let permissions = sqlx::query_scalar::<_, String>(
                "SELECT permission FROM project_role_permissions WHERE role_id = $1",
            )
            .bind(role.id)
            .fetch_all(&self.pool)
            .await
            .map_err(map_sqlx_error)?;

            result.push(ProjectRoleReadModel {
                id: ProjectRoleId::from_uuid(role.id),
                name: role.name,
                slug: ProjectRoleSlug::new(role.slug).expect("invalid slug in database"),
                permissions: permissions.into_iter().filter_map(|s| s.parse::<ProjectPermission>().ok()).collect(),
                is_default: role.is_default,
            });
        }

        Ok(result)
    }
}
