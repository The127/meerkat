use anyhow::Context;

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
    pub(crate) master_oidc_jwks_url: Option<String>,
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

        let master_oidc_jwks_url = std::env::var("MEERKAT_MASTER_OIDC_JWKS_URL").ok();

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
            master_oidc_jwks_url,
        })
    }
}
