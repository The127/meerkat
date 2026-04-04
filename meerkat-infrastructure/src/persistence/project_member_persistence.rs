use chrono::{DateTime, Utc};

use meerkat_application::error::ApplicationError;
use meerkat_domain::models::project_member::ProjectMember;

use super::error::map_sqlx_error;

pub(crate) struct ProjectMemberPersistence;

impl ProjectMemberPersistence {
    pub async fn insert(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        member: &ProjectMember,
        now: DateTime<Utc>,
    ) -> Result<(), ApplicationError> {
        sqlx::query(
            "INSERT INTO project_members (id, member_id, project_id, created_at) \
             VALUES ($1, $2, $3, $4)",
        )
        .bind(member.id().as_uuid())
        .bind(member.member_id().as_uuid())
        .bind(member.project_id().as_uuid())
        .bind(now)
        .execute(&mut **tx)
        .await
        .map_err(map_sqlx_error)?;

        for role_id in member.role_ids() {
            sqlx::query(
                "INSERT INTO project_member_roles (project_member_id, role_id) VALUES ($1, $2)",
            )
            .bind(member.id().as_uuid())
            .bind(role_id.as_uuid())
            .execute(&mut **tx)
            .await
            .map_err(map_sqlx_error)?;
        }

        Ok(())
    }
}
