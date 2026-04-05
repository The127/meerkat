use async_trait::async_trait;
use sqlx::PgPool;

use meerkat_application::error::ApplicationError;
use meerkat_application::ports::member_read_store::{ListMembersQuery, MemberReadModel, MemberReadStore};
use meerkat_application::ports::project_read_store::PagedResult;
use meerkat_domain::models::member::MemberId;
use meerkat_domain::models::org_role::OrgRole;

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
    last_seen: chrono::DateTime<chrono::Utc>,
    total: i64,
}

fn parse_org_roles(roles: Vec<String>) -> Vec<OrgRole> {
    roles.into_iter().filter_map(|s| s.parse::<OrgRole>().ok()).collect()
}

fn to_read_model(r: MemberRow) -> MemberReadModel {
    MemberReadModel {
        id: MemberId::from_uuid(r.id),
        sub: r.sub,
        preferred_name: r.preferred_name,
        org_roles: parse_org_roles(r.org_roles),
        created_at: r.created_at,
        last_seen: r.last_seen,
    }
}

#[async_trait]
impl MemberReadStore for PgMemberReadStore {
    async fn list_by_org(
        &self,
        query: &ListMembersQuery,
    ) -> Result<PagedResult<MemberReadModel>, ApplicationError> {
        let pattern = query.search.as_ref().map(|s| s.contains_pattern());
        let role_str = query.role.as_ref().map(|r| r.to_string());
        let slug_str = query.project_slug.as_ref().map(|s| s.as_str().to_string());

        let rows = sqlx::query_as::<_, MemberRow>(
            "SELECT id, sub, preferred_name, org_roles, created_at, last_seen, \
                    COUNT(*) OVER() AS total \
             FROM members \
             WHERE organization_id = $1 \
               AND ($4::text IS NULL OR preferred_name ILIKE $4 OR sub ILIKE $4) \
               AND ($5::text IS NULL OR $5 = ANY(org_roles)) \
               AND ($6::text IS NULL OR id IN ( \
                   SELECT pm.member_id FROM project_members pm \
                   JOIN projects p ON pm.project_id = p.id \
                   WHERE p.slug = $6 AND p.organization_id = $1 \
               )) \
             ORDER BY last_seen DESC \
             LIMIT $2 OFFSET $3",
        )
        .bind(query.org_id.as_uuid())
        .bind(query.limit)
        .bind(query.offset)
        .bind(pattern.as_deref())
        .bind(role_str.as_deref())
        .bind(slug_str.as_deref())
        .fetch_all(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        let total = rows.first().map(|r| r.total).unwrap_or(0);
        let items = rows.into_iter().map(to_read_model).collect();

        Ok(PagedResult { items, total })
    }

    async fn find_member_for_access(
        &self,
        member_id: &MemberId,
        org_id: &meerkat_domain::models::organization::OrganizationId,
    ) -> Result<Option<MemberReadModel>, ApplicationError> {
        #[derive(sqlx::FromRow)]
        struct SingleMemberRow {
            id: sqlx::types::Uuid,
            sub: String,
            preferred_name: String,
            org_roles: Vec<String>,
            created_at: chrono::DateTime<chrono::Utc>,
            last_seen: chrono::DateTime<chrono::Utc>,
        }

        let row = sqlx::query_as::<_, SingleMemberRow>(
            "SELECT id, sub, preferred_name, org_roles, created_at, last_seen \
             FROM members \
             WHERE id = $1 AND organization_id = $2",
        )
        .bind(member_id.as_uuid())
        .bind(org_id.as_uuid())
        .fetch_optional(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        Ok(row.map(|r| MemberReadModel {
            id: MemberId::from_uuid(r.id),
            sub: r.sub,
            preferred_name: r.preferred_name,
            org_roles: parse_org_roles(r.org_roles),
            created_at: r.created_at,
            last_seen: r.last_seen,
        }))
    }
}
