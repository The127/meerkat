use async_trait::async_trait;
use sqlx::PgPool;

use meerkat_application::error::ApplicationError;
use meerkat_application::ports::oidc_config_read_store::{OidcConfigReadModel, OidcConfigReadStore};
use meerkat_domain::models::oidc_config::{Audience, OidcConfigId};
use meerkat_domain::models::organization::OrganizationId;
use meerkat_domain::shared::url::Url;

use super::error::map_sqlx_error;

pub struct PgOidcConfigReadStore {
    pool: PgPool,
}

impl PgOidcConfigReadStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct OidcConfigRow {
    id: sqlx::types::Uuid,
    organization_id: sqlx::types::Uuid,
    issuer_url: String,
    audience: String,
    jwks_url: Option<String>,
}

impl From<OidcConfigRow> for OidcConfigReadModel {
    fn from(row: OidcConfigRow) -> Self {
        Self {
            id: OidcConfigId::from_uuid(row.id),
            organization_id: OrganizationId::from_uuid(row.organization_id),
            issuer_url: Url::new(row.issuer_url).expect("invalid issuer_url in database"),
            audience: Audience::new(row.audience).expect("invalid audience in database"),
            jwks_url: row.jwks_url.map(|u| Url::new(u).expect("invalid jwks_url in database")),
        }
    }
}

#[async_trait]
impl OidcConfigReadStore for PgOidcConfigReadStore {
    async fn find_active_by_org_id(
        &self,
        org_id: &OrganizationId,
    ) -> Result<OidcConfigReadModel, ApplicationError> {
        let row = sqlx::query_as::<_, OidcConfigRow>(
            "SELECT id, organization_id, issuer_url, audience, jwks_url \
             FROM oidc_configs \
             WHERE organization_id = $1 AND status = 'active'"
        )
        .bind(org_id.as_uuid())
        .fetch_one(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        Ok(row.into())
    }

    async fn find_by_issuer_and_audience(
        &self,
        issuer_url: &Url,
        audience: &Audience,
    ) -> Result<Option<OidcConfigReadModel>, ApplicationError> {
        let row = sqlx::query_as::<_, OidcConfigRow>(
            "SELECT id, organization_id, issuer_url, audience, jwks_url \
             FROM oidc_configs \
             WHERE issuer_url = $1 AND audience = $2 AND status = 'active'"
        )
        .bind(issuer_url.as_str())
        .bind(audience.as_str())
        .fetch_optional(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        Ok(row.map(Into::into))
    }
}

