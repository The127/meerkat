use anyhow::Context;
use sqlx::PgPool;
use tracing::info;

use meerkat_application::ports::organization_read_store::OrganizationReadStore;
use meerkat_application::ports::unit_of_work::UnitOfWorkFactory;
use meerkat_domain::models::oidc_config::{Audience, ClientId, OidcConfig};
use meerkat_domain::models::organization::{Organization, OrganizationSlug};
use meerkat_domain::ports::clock::Clock;
use meerkat_domain::shared::url::Url;
use meerkat_infrastructure::persistence::pg_organization_read_store::PgOrganizationReadStore;
use meerkat_infrastructure::persistence::pg_unit_of_work::PgUnitOfWorkFactory;

use crate::config::MeerkatConfig;

pub(crate) async fn bootstrap_master(
    config: &MeerkatConfig,
    pool: &PgPool,
    clock: &dyn Clock,
) -> anyhow::Result<()> {
    let read_store = PgOrganizationReadStore::new(pool.clone());

    let any_exists = read_store
        .any_exists()
        .await
        .context("Failed to check for existing organizations")?;

    if any_exists {
        info!("Organizations already exist, skipping bootstrap");
        return Ok(());
    }

    let slug = OrganizationSlug::new(&config.master_org_slug)
        .map_err(|e| anyhow::anyhow!("Invalid master organization slug: {e}"))?;

    let issuer_url =
        Url::new(&config.master_oidc_issuer_url).context("Invalid master OIDC issuer URL")?;

    let audience =
        Audience::new(&config.master_oidc_audience).context("Invalid master OIDC audience")?;

    let client_id =
        ClientId::new(&config.master_oidc_client_id).context("Invalid master OIDC client ID")?;

    let discovery_url = config
        .master_oidc_discovery_url
        .as_deref()
        .map(Url::new)
        .transpose()
        .context("Invalid master OIDC discovery URL")?;

    let oidc_config = OidcConfig::new(
        config.master_oidc_name.clone(),
        client_id,
        issuer_url,
        audience,
        discovery_url,
        clock,
    )
    .context("Failed to create master OIDC config")?;

    let org = Organization::new(
        config.master_org_name.clone(),
        slug,
        oidc_config,
        clock,
    )
    .context("Failed to create master organization")?;

    let uow_factory = PgUnitOfWorkFactory::new(pool.clone());
    let mut uow = uow_factory
        .create()
        .await
        .context("Failed to create unit of work")?;

    uow.organizations().add(org);
    uow.save_changes()
        .await
        .context("Failed to persist master organization")?;

    info!(
        name = config.master_org_name,
        slug = config.master_org_slug,
        "Bootstrapped master organization"
    );

    Ok(())
}
