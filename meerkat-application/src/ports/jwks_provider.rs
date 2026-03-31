use crate::error::ApplicationError;

#[async_trait::async_trait]
pub trait JwksProvider: Send + Sync {
    /// Resolve a signing key from a JWKS endpoint by `kid`.
    /// If `kid` is `None`, returns the first available key.
    /// Implementations handle caching and automatic retry on key miss (key rotation).
    async fn resolve_jwk(
        &self,
        jwks_url: &str,
        kid: Option<&str>,
    ) -> Result<serde_json::Value, ApplicationError>;
}
