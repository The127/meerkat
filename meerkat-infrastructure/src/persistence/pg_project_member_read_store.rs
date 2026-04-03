use async_trait::async_trait;
use sqlx::PgPool;

use meerkat_application::error::ApplicationError;
use meerkat_application::ports::project_member_read_store::{ProjectMemberReadModel, ProjectMemberReadStore};
use meerkat_domain::models::member::MemberId;
use meerkat_domain::models::project::ProjectId;
use meerkat_domain::models::project_role::ProjectRoleId;

use super::error::map_sqlx_error;

pub struct PgProjectMemberReadStore {
    pool: PgPool,
}

impl PgProjectMemberReadStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct ProjectMemberRow {
    member_id: sqlx::types::Uuid,
    preferred_name: String,
    sub: String,
    role_id: sqlx::types::Uuid,
    role_name: String,
    created_at: chrono::DateTime<chrono::Utc>,
}

#[async_trait]
impl ProjectMemberReadStore for PgProjectMemberReadStore {
    async fn list_by_project(
        &self,
        project_id: &ProjectId,
    ) -> Result<Vec<ProjectMemberReadModel>, ApplicationError> {
        let rows = sqlx::query_as::<_, ProjectMemberRow>(
            "SELECT m.id AS member_id, m.preferred_name, m.sub, \
                    pr.id AS role_id, pr.name AS role_name, \
                    pm.created_at \
             FROM project_members pm \
             JOIN members m ON m.id = pm.member_id \
             JOIN project_member_roles pmr ON pmr.project_member_id = pm.id \
             JOIN project_roles pr ON pr.id = pmr.role_id \
             WHERE pm.project_id = $1 \
             ORDER BY m.preferred_name",
        )
        .bind(project_id.as_uuid())
        .fetch_all(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        Ok(rows
            .into_iter()
            .map(|r| ProjectMemberReadModel {
                member_id: MemberId::from_uuid(r.member_id),
                preferred_name: r.preferred_name,
                sub: r.sub,
                role_id: ProjectRoleId::from_uuid(r.role_id),
                role_name: r.role_name,
                created_at: r.created_at,
            })
            .collect())
    }
}
