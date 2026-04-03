use async_trait::async_trait;
use sqlx::PgPool;

use meerkat_application::error::ApplicationError;
use meerkat_application::ports::member_read_store::{MemberReadModel, MemberReadStore};
use meerkat_domain::models::member::MemberId;
use meerkat_domain::models::org_role::OrgRole;
use meerkat_domain::models::organization::OrganizationId;

use super::error::map_sqlx_error;

pub struct PgMemberReadStore {
    pool: PgPool,
}

impl PgMemberReadStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct MemberRow {
    id: sqlx::types::Uuid,
    sub: String,
    preferred_name: String,
    org_roles: Vec<String>,
    created_at: chrono::DateTime<chrono::Utc>,
}

#[async_trait]
impl MemberReadStore for PgMemberReadStore {
    async fn list_by_org(
        &self,
        org_id: &OrganizationId,
    ) -> Result<Vec<MemberReadModel>, ApplicationError> {
        let rows = sqlx::query_as::<_, MemberRow>(
            "SELECT id, sub, preferred_name, org_roles, created_at \
             FROM members \
             WHERE organization_id = $1 \
             ORDER BY created_at",
        )
        .bind(org_id.as_uuid())
        .fetch_all(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        Ok(rows
            .into_iter()
            .map(|r| MemberReadModel {
                id: MemberId::from_uuid(r.id),
                sub: r.sub,
                preferred_name: r.preferred_name,
                org_roles: r.org_roles.into_iter().filter_map(|s| s.parse::<OrgRole>().ok()).collect(),
                created_at: r.created_at,
            })
            .collect())
    }
}
