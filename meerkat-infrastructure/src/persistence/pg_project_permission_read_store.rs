use async_trait::async_trait;
use sqlx::PgPool;

use meerkat_application::error::ApplicationError;
use meerkat_application::ports::project_permission_read_store::ProjectPermissionReadStore;
use meerkat_domain::models::member::MemberId;
use meerkat_domain::models::permission::ProjectPermission;
use meerkat_domain::models::project::ProjectId;

use super::error::map_sqlx_error;

pub struct PgProjectPermissionReadStore {
    pool: PgPool,
}

impl PgProjectPermissionReadStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ProjectPermissionReadStore for PgProjectPermissionReadStore {
    async fn get_member_permissions(
        &self,
        member_id: &MemberId,
        project_id: &ProjectId,
    ) -> Result<Vec<ProjectPermission>, ApplicationError> {
        let rows = sqlx::query_scalar::<_, String>(
            "SELECT DISTINCT prp.permission \
             FROM project_members pm \
             JOIN project_member_roles pmr ON pmr.project_member_id = pm.id \
             JOIN project_role_permissions prp ON prp.role_id = pmr.role_id \
             WHERE pm.member_id = $1 AND pm.project_id = $2",
        )
        .bind(member_id.as_uuid())
        .bind(project_id.as_uuid())
        .fetch_all(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        let permissions = rows
            .into_iter()
            .filter_map(|s| s.parse::<ProjectPermission>().ok())
            .collect();

        Ok(permissions)
    }
}
