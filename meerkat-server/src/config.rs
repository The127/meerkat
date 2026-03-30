pub(crate) struct MeerkatConfig {
    pub(crate) listen_addr: String,
}

impl MeerkatConfig {
    pub(crate) fn from_env() -> anyhow::Result<Self> {
        let listen_addr =
            std::env::var("MEERKAT_LISTEN_ADDR").unwrap_or_else(|_| "0.0.0.0:3030".to_string());

        Ok(Self { listen_addr })
    }
}
