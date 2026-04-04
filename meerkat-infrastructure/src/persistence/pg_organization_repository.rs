use async_trait::async_trait;
use sqlx::PgPool;

use meerkat_application::error::ApplicationError;
use meerkat_application::ports::organization_repository::OrganizationRepository;
use vec1::Vec1;
use meerkat_domain::models::oidc_config::{
    Audience, ClaimMapping, ClientId, OidcConfig, OidcConfigId, OidcConfigState, OidcConfigStatus, RoleValues, Url,
};
use meerkat_domain::models::organization::{
    Organization, OrganizationId, OrganizationIdentifier, OrganizationSlug, OrganizationState,
};
use meerkat_domain::shared::version::Version;

use super::change_buffer::ChangeTracker;
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
    tracker: ChangeTracker<OrganizationId, Organization, OrgEntry>,
}

impl PgOrganizationRepository {
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool,
            tracker: ChangeTracker::new(),
        }
    }

    pub(crate) fn take_entries(&self) -> Vec<OrgEntry> {
        self.tracker.take_entries()
    }
}

#[async_trait]
impl OrganizationRepository for PgOrganizationRepository {
    fn add(&self, org: Organization) {
        self.tracker.push(OrgEntry::Added(org));
    }

    fn save(&self, org: Organization) {
        let snapshot = self.tracker.take_snapshot(org.id());
        self.tracker.push(OrgEntry::Modified { entity: org, snapshot });
    }

    fn delete(&self, id: OrganizationId) {
        self.tracker.remove_snapshot(&id);
        self.tracker.push(OrgEntry::Deleted(id));
    }

    async fn find(&self, identifier: &OrganizationIdentifier) -> Result<Organization, ApplicationError> {
        let row = match identifier {
            OrganizationIdentifier::Id(id) => {
                sqlx::query_as::<_, OrgRow>(
                    "SELECT id, name, slug, version FROM organizations WHERE id = $1",
                )
                .bind(id.as_uuid())
                .fetch_optional(&self.pool)
                .await
            }
            OrganizationIdentifier::Slug(slug) => {
                sqlx::query_as::<_, OrgRow>(
                    "SELECT id, name, slug, version FROM organizations WHERE slug = $1",
                )
                .bind(slug.as_str())
                .fetch_optional(&self.pool)
                .await
            }
        }
        .map_err(map_sqlx_error)?
        .ok_or(ApplicationError::NotFound)?;

        let org_id = row.id;

        let config_rows = sqlx::query_as::<_, OidcConfigRow>(
            "SELECT id, name, client_id, issuer_url, audience, discovery_url, \
             sub_claim, name_claim, role_claim, owner_values, admin_values, member_values, \
             status \
             FROM oidc_configs WHERE organization_id = $1",
        )
        .bind(org_id)
        .fetch_all(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        let oidc_configs: Vec<OidcConfig> = config_rows
            .into_iter()
            .map(|r| {
                let claim_mapping = ClaimMapping::new(
                    r.sub_claim.expect("missing sub_claim in database"),
                    r.name_claim.expect("missing name_claim in database"),
                    r.role_claim.expect("missing role_claim in database"),
                    RoleValues::new(
                        Vec1::try_from_vec(r.owner_values.unwrap_or_default()).expect("empty owner_values in database"),
                        Vec1::try_from_vec(r.admin_values.unwrap_or_default()).expect("empty admin_values in database"),
                        Vec1::try_from_vec(r.member_values.unwrap_or_default()).expect("empty member_values in database"),
                    ),
                ).expect("invalid claim_mapping in database");

                OidcConfig::reconstitute(OidcConfigState {
                    id: OidcConfigId::from_uuid(r.id),
                    name: r.name,
                    client_id: ClientId::new(r.client_id).expect("invalid client_id in database"),
                    issuer_url: Url::new(r.issuer_url).expect("invalid issuer_url in database"),
                    audience: Audience::new(r.audience).expect("invalid audience in database"),
                    discovery_url: r.discovery_url.map(|u| Url::new(u).expect("invalid discovery_url in database")),
                    claim_mapping,
                    status: r.status.parse::<OidcConfigStatus>().expect("invalid status in database"),
                })
            })
            .collect();

        let org = Organization::reconstitute(OrganizationState {
            id: OrganizationId::from_uuid(row.id),
            name: row.name,
            slug: OrganizationSlug::new(row.slug).expect("invalid slug in database"),
            oidc_configs,
            version: Version::new(row.version as u64),
        });

        self.tracker.track(org.id().clone(), org.clone());

        Ok(org)
    }
}

#[derive(sqlx::FromRow)]
struct OrgRow {
    id: sqlx::types::Uuid,
    name: String,
    slug: String,
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
    sub_claim: Option<String>,
    name_claim: Option<String>,
    role_claim: Option<String>,
    owner_values: Option<Vec<String>>,
    admin_values: Option<Vec<String>>,
    member_values: Option<Vec<String>>,
    status: String,
}

