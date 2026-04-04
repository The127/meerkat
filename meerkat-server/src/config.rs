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
    pub(crate) master_oidc: MasterOidcConfig,
}

pub(crate) struct MasterOidcConfig {
    pub(crate) name: String,
    pub(crate) client_id: String,
    pub(crate) issuer_url: String,
    pub(crate) audience: String,
    pub(crate) discovery_url: Option<String>,
    pub(crate) sub_claim: String,
    pub(crate) name_claim: String,
    pub(crate) role_claim: String,
    pub(crate) role_values: RoleValues,
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

        let master_oidc = MasterOidcConfig::from_env()?;

        Ok(Self {
            database_url,
            listen_addr,
            base_domain,
            master_org_name,
            master_org_slug,
            master_oidc,
        })
    }
}

impl MasterOidcConfig {
    fn from_env() -> anyhow::Result<Self> {
        let name = std::env::var("MEERKAT_MASTER_OIDC_NAME")
            .unwrap_or_else(|_| "Default".to_string());

        let client_id = std::env::var("MEERKAT_MASTER_OIDC_CLIENT_ID")
            .context("MEERKAT_MASTER_OIDC_CLIENT_ID environment variable must be set")?;

        let issuer_url = std::env::var("MEERKAT_MASTER_OIDC_ISSUER_URL")
            .context("MEERKAT_MASTER_OIDC_ISSUER_URL environment variable must be set")?;

        let audience = std::env::var("MEERKAT_MASTER_OIDC_AUDIENCE")
            .context("MEERKAT_MASTER_OIDC_AUDIENCE environment variable must be set")?;

        let discovery_url = std::env::var("MEERKAT_MASTER_OIDC_DISCOVERY_URL").ok();

        let sub_claim = std::env::var("MEERKAT_MASTER_OIDC_SUB_CLAIM")
            .unwrap_or_else(|_| "sub".to_string());

        let name_claim = std::env::var("MEERKAT_MASTER_OIDC_NAME_CLAIM")
            .unwrap_or_else(|_| "preferred_username".to_string());

        let role_claim = std::env::var("MEERKAT_MASTER_OIDC_ROLE_CLAIM")
            .unwrap_or_else(|_| "roles".to_string());

        let owner_values = Vec1::try_from_vec(parse_csv_env("MEERKAT_MASTER_OIDC_OWNER_VALUES"))
            .map_err(|_| anyhow::anyhow!("MEERKAT_MASTER_OIDC_OWNER_VALUES must be set with at least one comma-separated value"))?;

        let admin_values = Vec1::try_from_vec(parse_csv_env("MEERKAT_MASTER_OIDC_ADMIN_VALUES"))
            .map_err(|_| anyhow::anyhow!("MEERKAT_MASTER_OIDC_ADMIN_VALUES must be set with at least one comma-separated value"))?;

        let member_values = Vec1::try_from_vec(parse_csv_env("MEERKAT_MASTER_OIDC_MEMBER_VALUES"))
            .map_err(|_| anyhow::anyhow!("MEERKAT_MASTER_OIDC_MEMBER_VALUES must be set with at least one comma-separated value"))?;

        let role_values = RoleValues::new(
            owner_values,
            admin_values,
            member_values,
        );

        Ok(Self {
            name,
            client_id,
            issuer_url,
            audience,
            discovery_url,
            sub_claim,
            name_claim,
            role_claim,
            role_values,
        })
    }
}
