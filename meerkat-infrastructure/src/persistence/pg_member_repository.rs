use async_trait::async_trait;
use sqlx::PgPool;

use meerkat_application::error::ApplicationError;
use meerkat_application::ports::member_repository::MemberRepository;
use meerkat_domain::models::member::{MemberId, Sub};
use meerkat_domain::models::organization::OrganizationId;

use super::error::map_sqlx_error;

pub struct PgMemberRepository {
    pool: PgPool,
}

impl PgMemberRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl MemberRepository for PgMemberRepository {
    async fn find_or_create(
        &self,
        org_id: &OrganizationId,
        sub: &Sub,
        preferred_name: &str,
    ) -> Result<MemberId, ApplicationError> {
        let row = sqlx::query_scalar::<_, sqlx::types::Uuid>(
            "INSERT INTO members (id, organization_id, sub, preferred_name, created_at, updated_at) \
             VALUES ($1, $2, $3, $4, now(), now()) \
             ON CONFLICT (organization_id, sub) \
             DO UPDATE SET preferred_name = EXCLUDED.preferred_name, updated_at = now() \
             RETURNING id",
        )
        .bind(sqlx::types::Uuid::new_v4())
        .bind(org_id.as_uuid())
        .bind(sub.as_str())
        .bind(preferred_name)
        .fetch_one(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        Ok(MemberId::from_uuid(row))
    }
}
