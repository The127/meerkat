use std::sync::Arc;

use async_trait::async_trait;
use sqlx::PgPool;

use meerkat_application::error::ApplicationError;
use raccoon_clock_rs::Clock;
use meerkat_application::ports::member_repository::MemberRepository;
use meerkat_domain::models::member::{MemberId, Sub};
use meerkat_domain::models::org_role::OrgRole;
use meerkat_domain::models::organization::OrganizationId;

use super::error::map_sqlx_error;

pub struct PgMemberRepository {
    pool: PgPool,
    clock: Arc<dyn Clock>,
}

impl PgMemberRepository {
    pub fn new(pool: PgPool, clock: Arc<dyn Clock>) -> Self {
        Self { pool, clock }
    }
}

#[async_trait]
impl MemberRepository for PgMemberRepository {
    async fn find_or_create(
        &self,
        org_id: &OrganizationId,
        sub: &Sub,
        preferred_name: &str,
        org_roles: &[OrgRole],
    ) -> Result<MemberId, ApplicationError> {
        let role_strings: Vec<String> = org_roles.iter().map(|r| r.to_string()).collect();
        let now = self.clock.now();

        let row = sqlx::query_scalar::<_, sqlx::types::Uuid>(
            "INSERT INTO members (id, organization_id, sub, preferred_name, org_roles, created_at, updated_at, last_seen) \
             VALUES ($1, $2, $3, $4, $5, $6, $6, $6) \
             ON CONFLICT (organization_id, sub) \
             DO UPDATE SET preferred_name = EXCLUDED.preferred_name, org_roles = EXCLUDED.org_roles, updated_at = $6 \
             RETURNING id",
        )
        .bind(sqlx::types::Uuid::new_v4())
        .bind(org_id.as_uuid())
        .bind(sub.as_str())
        .bind(preferred_name)
        .bind(&role_strings)
        .bind(now)
        .fetch_one(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        Ok(MemberId::from_uuid(row))
    }

    async fn touch_last_seen(&self, member_id: &MemberId) -> Result<(), ApplicationError> {
        let now = self.clock.now();

        sqlx::query(
            "UPDATE members SET last_seen = $2 \
             WHERE id = $1 AND (last_seen IS NULL OR last_seen < $2 - interval '15 minutes')",
        )
        .bind(member_id.as_uuid())
        .bind(now)
        .execute(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        Ok(())
    }
}
