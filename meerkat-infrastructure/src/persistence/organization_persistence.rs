use std::collections::{HashMap, HashSet};

use chrono::{DateTime, Utc};

use meerkat_application::error::ApplicationError;
use meerkat_domain::models::oidc_config::OidcConfig;
use meerkat_domain::models::organization::{Organization, OrganizationId};

use super::error::map_sqlx_error;

pub(crate) struct OrganizationPersistence;

impl OrganizationPersistence {
    pub async fn insert(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        org: &Organization,
        now: DateTime<Utc>,
    ) -> Result<(), ApplicationError> {
        sqlx::query(
            "INSERT INTO organizations (id, name, slug, created_at, updated_at, version) \
             VALUES ($1, $2, $3, $4, $5, $6)",
        )
        .bind(org.id().as_uuid())
        .bind(org.name())
        .bind(org.slug().as_str())
        .bind(now)
        .bind(now)
        .bind(org.version().as_u64() as i64)
        .execute(&mut **tx)
        .await
        .map_err(map_sqlx_error)?;

        for config in org.oidc_configs() {
            Self::insert_oidc_config(tx, org.id(), config, now).await?;
        }

        Ok(())
    }

    pub async fn update(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        org: &Organization,
        snapshot: &Organization,
        now: DateTime<Utc>,
    ) -> Result<(), ApplicationError> {
        // Diff oidc_configs collection
        let snapshot_configs: HashMap<_, _> = snapshot
            .oidc_configs()
            .iter()
            .map(|c| (c.id().clone(), c))
            .collect();

        let mut current_ids = HashSet::new();
        let mut has_child_changes = false;

        for config in org.oidc_configs() {
            current_ids.insert(config.id().clone());

            match snapshot_configs.get(config.id()) {
                None => {
                    Self::insert_oidc_config(tx, org.id(), config, now).await?;
                    has_child_changes = true;
                }
                Some(old) => {
                    if oidc_config_changed(config, old) {
                        Self::update_oidc_config(tx, config, now).await?;
                        has_child_changes = true;
                    }
                }
            }
        }

        for snapshot_id in snapshot_configs.keys() {
            if !current_ids.contains(snapshot_id) {
                Self::delete_oidc_config(tx, snapshot_id).await?;
                has_child_changes = true;
            }
        }

        let org_row_changed = org.name() != snapshot.name()
            || org.slug() != snapshot.slug();

        if org_row_changed || has_child_changes {
            let new_version = snapshot.version().increment();

            let result = sqlx::query(
                "UPDATE organizations SET name = $1, slug = $2, updated_at = $3, version = $4 \
                 WHERE id = $5 AND version = $6",
            )
            .bind(org.name())
            .bind(org.slug().as_str())
            .bind(now)
            .bind(new_version.as_u64() as i64)
            .bind(org.id().as_uuid())
            .bind(snapshot.version().as_u64() as i64)
            .execute(&mut **tx)
            .await
            .map_err(map_sqlx_error)?;

            if result.rows_affected() == 0 {
                return Err(ApplicationError::Conflict);
            }
        }

        Ok(())
    }

    pub async fn delete(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        id: &OrganizationId,
    ) -> Result<(), ApplicationError> {
        sqlx::query("DELETE FROM organizations WHERE id = $1")
            .bind(id.as_uuid())
            .execute(&mut **tx)
            .await
            .map_err(map_sqlx_error)?;

        Ok(())
    }

    async fn insert_oidc_config(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        org_id: &OrganizationId,
        config: &OidcConfig,
        now: DateTime<Utc>,
    ) -> Result<(), ApplicationError> {
        sqlx::query(
            "INSERT INTO oidc_configs (id, organization_id, name, client_id, issuer_url, audience, discovery_url, \
             sub_claim, name_claim, role_claim, owner_values, admin_values, member_values, \
             status, created_at, updated_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16)",
        )
        .bind(config.id().as_uuid())
        .bind(org_id.as_uuid())
        .bind(config.name())
        .bind(config.client_id().as_str())
        .bind(config.issuer_url().as_str())
        .bind(config.audience().as_str())
        .bind(config.discovery_url().map(|u| u.as_str().to_string()))
        .bind(config.claim_mapping().sub_claim().as_str())
        .bind(config.claim_mapping().name_claim().as_str())
        .bind(config.claim_mapping().role_claim().as_str())
        .bind(config.claim_mapping().owner_values().as_slice())
        .bind(config.claim_mapping().admin_values().as_slice())
        .bind(config.claim_mapping().member_values().as_slice())
        .bind(config.status().as_ref())
        .bind(now)
        .bind(now)
        .execute(&mut **tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(())
    }

    async fn update_oidc_config(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        config: &OidcConfig,
        now: DateTime<Utc>,
    ) -> Result<(), ApplicationError> {
        sqlx::query(
            "UPDATE oidc_configs SET name = $1, client_id = $2, issuer_url = $3, audience = $4, \
             discovery_url = $5, sub_claim = $6, name_claim = $7, role_claim = $8, \
             owner_values = $9, admin_values = $10, member_values = $11, \
             status = $12, updated_at = $13 WHERE id = $14",
        )
        .bind(config.name())
        .bind(config.client_id().as_str())
        .bind(config.issuer_url().as_str())
        .bind(config.audience().as_str())
        .bind(config.discovery_url().map(|u| u.as_str().to_string()))
        .bind(config.claim_mapping().sub_claim().as_str())
        .bind(config.claim_mapping().name_claim().as_str())
        .bind(config.claim_mapping().role_claim().as_str())
        .bind(config.claim_mapping().owner_values().as_slice())
        .bind(config.claim_mapping().admin_values().as_slice())
        .bind(config.claim_mapping().member_values().as_slice())
        .bind(config.status().as_ref())
        .bind(now)
        .bind(config.id().as_uuid())
        .execute(&mut **tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(())
    }

    async fn delete_oidc_config(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        id: &meerkat_domain::models::oidc_config::OidcConfigId,
    ) -> Result<(), ApplicationError> {
        sqlx::query("DELETE FROM oidc_configs WHERE id = $1")
            .bind(id.as_uuid())
            .execute(&mut **tx)
            .await
            .map_err(map_sqlx_error)?;

        Ok(())
    }
}

fn oidc_config_changed(current: &OidcConfig, snapshot: &OidcConfig) -> bool {
    current.name() != snapshot.name()
        || current.client_id() != snapshot.client_id()
        || current.issuer_url() != snapshot.issuer_url()
        || current.audience() != snapshot.audience()
        || current.discovery_url() != snapshot.discovery_url()
        || current.claim_mapping() != snapshot.claim_mapping()
        || current.status() != snapshot.status()
}
