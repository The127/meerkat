use chrono::{DateTime, Utc};

use meerkat_application::error::ApplicationError;
use meerkat_domain::models::project_role::{ProjectRole, ProjectRoleId};

use super::error::map_sqlx_error;

pub(crate) struct ProjectRolePersistence;

impl ProjectRolePersistence {
    pub async fn insert(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        role: &ProjectRole,
        now: DateTime<Utc>,
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
        .bind(now)
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

    pub async fn update(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        role: &ProjectRole,
        _snapshot: &ProjectRole,
        now: DateTime<Utc>,
    ) -> Result<(), ApplicationError> {
        sqlx::query(
            "UPDATE project_roles SET name = $1, slug = $2, updated_at = $3 WHERE id = $4",
        )
        .bind(role.name())
        .bind(role.slug().as_str())
        .bind(now)
        .bind(role.id().as_uuid())
        .execute(&mut **tx)
        .await
        .map_err(map_sqlx_error)?;

        sqlx::query("DELETE FROM project_role_permissions WHERE role_id = $1")
            .bind(role.id().as_uuid())
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

    pub async fn delete(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        id: &ProjectRoleId,
    ) -> Result<(), ApplicationError> {
        sqlx::query("DELETE FROM project_roles WHERE id = $1")
            .bind(id.as_uuid())
            .execute(&mut **tx)
            .await
            .map_err(map_sqlx_error)?;

        Ok(())
    }
}
