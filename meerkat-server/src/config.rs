use anyhow::Context;
use meerkat_domain::models::oidc_config::RoleValues;
use vec1::Vec1;

fn parse_csv_env(name: &str) -> Vec<String> {
    std::env::var(name)
        .map(|v| v.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect())
        .unwrap_or_default()
}

pub(crate) struct MeerkatConfig {
    pub(crate) database_url: String,
    pub(crate) listen_addr: String,
    pub(crate) base_domain: String,
    pub(crate) master_org_name: String,
    pub(crate) master_org_slug: String,
    pub(crate) master_oidc_name: String,
    pub(crate) master_oidc_client_id: String,
    pub(crate) master_oidc_issuer_url: String,
    pub(crate) master_oidc_audience: String,
    pub(crate) master_oidc_discovery_url: Option<String>,
    pub(crate) master_oidc_sub_claim: String,
    pub(crate) master_oidc_name_claim: String,
    pub(crate) master_oidc_role_claim: String,
    pub(crate) master_oidc_role_values: meerkat_domain::models::oidc_config::RoleValues,
}

impl MeerkatConfig {
    pub(crate) fn from_env() -> anyhow::Result<Self> {
        let database_url = std::env::var("MEERKAT_DATABASE_URL")
            .context("MEERKAT_DATABASE_URL environment variable must be set")?;

        let listen_addr =
            std::env::var("MEERKAT_LISTEN_ADDR").unwrap_or_else(|_| "0.0.0.0:3030".to_string());

        let base_domain = std::env::var("MEERKAT_BASE_DOMAIN")
            .context("MEERKAT_BASE_DOMAIN environment variable must be set")?;

        let master_org_name = std::env::var("MEERKAT_MASTER_ORG_NAME")
            .context("MEERKAT_MASTER_ORG_NAME environment variable must be set")?;

        let master_org_slug = std::env::var("MEERKAT_MASTER_ORG_SLUG")
            .context("MEERKAT_MASTER_ORG_SLUG environment variable must be set")?;

        let master_oidc_name = std::env::var("MEERKAT_MASTER_OIDC_NAME")
            .unwrap_or_else(|_| "Default".to_string());

        let master_oidc_client_id = std::env::var("MEERKAT_MASTER_OIDC_CLIENT_ID")
            .context("MEERKAT_MASTER_OIDC_CLIENT_ID environment variable must be set")?;

        let master_oidc_issuer_url = std::env::var("MEERKAT_MASTER_OIDC_ISSUER_URL")
            .context("MEERKAT_MASTER_OIDC_ISSUER_URL environment variable must be set")?;

        let master_oidc_audience = std::env::var("MEERKAT_MASTER_OIDC_AUDIENCE")
            .context("MEERKAT_MASTER_OIDC_AUDIENCE environment variable must be set")?;

        let master_oidc_discovery_url = std::env::var("MEERKAT_MASTER_OIDC_DISCOVERY_URL").ok();

        let master_oidc_sub_claim = std::env::var("MEERKAT_MASTER_OIDC_SUB_CLAIM")
            .unwrap_or_else(|_| "sub".to_string());

        let master_oidc_name_claim = std::env::var("MEERKAT_MASTER_OIDC_NAME_CLAIM")
            .unwrap_or_else(|_| "preferred_username".to_string());

        let master_oidc_role_claim = std::env::var("MEERKAT_MASTER_OIDC_ROLE_CLAIM")
            .unwrap_or_else(|_| "roles".to_string());

        let master_oidc_owner_values = Vec1::try_from_vec(parse_csv_env("MEERKAT_MASTER_OIDC_OWNER_VALUES"))
            .map_err(|_| anyhow::anyhow!("MEERKAT_MASTER_OIDC_OWNER_VALUES must be set with at least one comma-separated value"))?;

        let master_oidc_admin_values = Vec1::try_from_vec(parse_csv_env("MEERKAT_MASTER_OIDC_ADMIN_VALUES"))
            .map_err(|_| anyhow::anyhow!("MEERKAT_MASTER_OIDC_ADMIN_VALUES must be set with at least one comma-separated value"))?;

        let master_oidc_member_values = Vec1::try_from_vec(parse_csv_env("MEERKAT_MASTER_OIDC_MEMBER_VALUES"))
            .map_err(|_| anyhow::anyhow!("MEERKAT_MASTER_OIDC_MEMBER_VALUES must be set with at least one comma-separated value"))?;

        let master_oidc_role_values = RoleValues::new(
            master_oidc_owner_values,
            master_oidc_admin_values,
            master_oidc_member_values,
        );

        Ok(Self {
            database_url,
            listen_addr,
            base_domain,
            master_org_name,
            master_org_slug,
            master_oidc_name,
            master_oidc_client_id,
            master_oidc_issuer_url,
            master_oidc_audience,
            master_oidc_discovery_url,
            master_oidc_sub_claim,
            master_oidc_name_claim,
            master_oidc_role_claim,
            master_oidc_role_values,
        })
    }
}
