use meerkat_application::error::ApplicationError;
use meerkat_domain::models::project_role::ProjectRole;

use super::error::map_sqlx_error;

pub(crate) struct ProjectRolePersistence;

impl ProjectRolePersistence {
    pub async fn insert(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        role: &ProjectRole,
    ) -> Result<(), ApplicationError> {
        sqlx::query(
            "INSERT INTO project_roles (id, project_id, name, slug, is_default, created_at) \
             VALUES ($1, $2, $3, $4, $5, $6)",
        )
        .bind(role.id().as_uuid())
        .bind(role.project_id().as_uuid())
        .bind(role.name())
        .bind(role.slug().as_str())
        .bind(role.is_default())
        .bind(role.created_at())
        .execute(&mut **tx)
        .await
        .map_err(map_sqlx_error)?;

        for permission in role.permissions().iter() {
            sqlx::query(
                "INSERT INTO project_role_permissions (role_id, permission) VALUES ($1, $2)",
            )
            .bind(role.id().as_uuid())
            .bind(permission.as_ref())
            .execute(&mut **tx)
            .await
            .map_err(map_sqlx_error)?;
        }

        Ok(())
    }
}
