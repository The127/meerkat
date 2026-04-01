use std::collections::HashMap;
use std::sync::Mutex;

use async_trait::async_trait;
use sqlx::PgPool;

use meerkat_application::error::ApplicationError;
use meerkat_application::ports::organization_repository::OrganizationRepository;
use meerkat_domain::models::oidc_config::{
    Audience, ClientId, OidcConfig, OidcConfigId, OidcConfigState, OidcConfigStatus, Url,
};
use meerkat_domain::models::organization::{
    Organization, OrganizationId, OrganizationSlug, OrganizationState,
};
use meerkat_domain::shared::version::Version;

use super::error::map_sqlx_error;

pub(crate) enum OrgEntry {
    Added(Organization),
    Modified {
        entity: Organization,
        snapshot: Organization,
    },
    Deleted(OrganizationId),
}

pub struct PgOrganizationRepository {
    pool: PgPool,
    snapshots: Mutex<HashMap<OrganizationId, Organization>>,
    buffer: Mutex<Vec<OrgEntry>>,
}

impl PgOrganizationRepository {
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool,
            snapshots: Mutex::new(HashMap::new()),
            buffer: Mutex::new(Vec::new()),
        }
    }

    pub(crate) fn take_entries(&self) -> Vec<OrgEntry> {
        std::mem::take(&mut *self.buffer.lock().unwrap())
    }
}

#[async_trait]
impl OrganizationRepository for PgOrganizationRepository {
    fn add(&self, org: Organization) {
        self.buffer.lock().unwrap().push(OrgEntry::Added(org));
    }

    fn save(&self, org: Organization) {
        let snapshot = self
            .snapshots
            .lock()
            .unwrap()
            .remove(org.id())
            .expect("save called without prior find_by_id");

        self.buffer
            .lock()
            .unwrap()
            .push(OrgEntry::Modified { entity: org, snapshot });
    }

    fn delete(&self, id: OrganizationId) {
        self.snapshots.lock().unwrap().remove(&id);
        self.buffer.lock().unwrap().push(OrgEntry::Deleted(id));
    }

    async fn find_by_id(&self, id: &OrganizationId) -> Result<Organization, ApplicationError> {
        let row = sqlx::query_as::<_, OrgRow>(
            "SELECT id, name, slug, created_at, updated_at, version FROM organizations WHERE id = $1",
        )
        .bind(id.as_uuid())
        .fetch_optional(&self.pool)
        .await
        .map_err(map_sqlx_error)?
        .ok_or(ApplicationError::NotFound)?;

        let config_rows = sqlx::query_as::<_, OidcConfigRow>(
            "SELECT id, name, client_id, issuer_url, audience, discovery_url, status, created_at, updated_at \
             FROM oidc_configs WHERE organization_id = $1",
        )
        .bind(id.as_uuid())
        .fetch_all(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        let oidc_configs: Vec<OidcConfig> = config_rows
            .into_iter()
            .map(|r| {
                OidcConfig::reconstitute(OidcConfigState {
                    id: OidcConfigId::from_uuid(r.id),
                    name: r.name,
                    client_id: ClientId::new(r.client_id).expect("invalid client_id in database"),
                    issuer_url: Url::new(r.issuer_url).expect("invalid issuer_url in database"),
                    audience: Audience::new(r.audience).expect("invalid audience in database"),
                    discovery_url: r.discovery_url.map(|u| Url::new(u).expect("invalid discovery_url in database")),
                    status: r.status.parse::<OidcConfigStatus>().expect("invalid status in database"),
                    created_at: r.created_at,
                    updated_at: r.updated_at,
                })
            })
            .collect();

        let org = Organization::reconstitute(OrganizationState {
            id: OrganizationId::from_uuid(row.id),
            name: row.name,
            slug: OrganizationSlug::new(row.slug).expect("invalid slug in database"),
            oidc_configs,
            created_at: row.created_at,
            updated_at: row.updated_at,
            version: Version::new(row.version as u64),
        });

        self.snapshots
            .lock()
            .unwrap()
            .insert(id.clone(), org.clone());

        Ok(org)
    }
}

#[derive(sqlx::FromRow)]
struct OrgRow {
    id: sqlx::types::Uuid,
    name: String,
    slug: String,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
    version: i64,
}

#[derive(sqlx::FromRow)]
struct OidcConfigRow {
    id: sqlx::types::Uuid,
    name: String,
    client_id: String,
    issuer_url: String,
    audience: String,
    discovery_url: Option<String>,
    status: String,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

