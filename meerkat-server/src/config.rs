use anyhow::Context;

pub(crate) struct MeerkatConfig {
    pub(crate) database_url: String,
    pub(crate) listen_addr: String,
}

impl MeerkatConfig {
    pub(crate) fn from_env() -> anyhow::Result<Self> {
        let database_url = std::env::var("MEERKAT_DATABASE_URL")
            .context("MEERKAT_DATABASE_URL environment variable must be set")?;

        let listen_addr =
            std::env::var("MEERKAT_LISTEN_ADDR").unwrap_or_else(|_| "0.0.0.0:3030".to_string());

        Ok(Self {
            database_url,
            listen_addr,
        })
    }
}
