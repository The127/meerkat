use crate::error::ApplicationError;

#[async_trait::async_trait]
pub trait OidcDiscoveryProvider: Send + Sync {
    /// Resolves the JWKS URI from an OIDC discovery document.
    /// Implementations should cache the discovery document.
    async fn resolve_jwks_uri(
        &self,
        discovery_url: &str,
    ) -> Result<String, ApplicationError>;
}
