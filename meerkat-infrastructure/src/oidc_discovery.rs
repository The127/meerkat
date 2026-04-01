use std::collections::HashMap;
use std::time::{Duration, Instant};

use async_trait::async_trait;
use tokio::sync::RwLock;

use meerkat_application::error::ApplicationError;
use meerkat_application::ports::oidc_discovery_provider::OidcDiscoveryProvider;

struct CachedDiscovery {
    jwks_uri: String,
    expires_at: Instant,
}

pub struct CachedOidcDiscoveryProvider {
    client: reqwest::Client,
    cache: RwLock<HashMap<String, CachedDiscovery>>,
    ttl: Duration,
}

impl CachedOidcDiscoveryProvider {
    pub fn new(ttl: Duration) -> Self {
        Self {
            client: reqwest::Client::new(),
            cache: RwLock::new(HashMap::new()),
            ttl,
        }
    }
}

#[async_trait]
impl OidcDiscoveryProvider for CachedOidcDiscoveryProvider {
    async fn resolve_jwks_uri(
        &self,
        discovery_url: &str,
    ) -> Result<String, ApplicationError> {
        // Check cache first
        {
            let cache = self.cache.read().await;
            if let Some(entry) = cache.get(discovery_url)
                && entry.expires_at > Instant::now()
            {
                return Ok(entry.jwks_uri.clone());
            }
        }

        // Fetch discovery document
        let json = self
            .client
            .get(discovery_url)
            .send()
            .await
            .map_err(|e| ApplicationError::Internal(format!("OIDC discovery fetch failed: {e}")))?
            .error_for_status()
            .map_err(|e| ApplicationError::Internal(format!("OIDC discovery fetch failed: {e}")))?
            .text()
            .await
            .map_err(|e| ApplicationError::Internal(format!("OIDC discovery read failed: {e}")))?;

        let doc: serde_json::Value = serde_json::from_str(&json)
            .map_err(|e| ApplicationError::Internal(format!("invalid OIDC discovery JSON: {e}")))?;

        let jwks_uri = doc
            .get("jwks_uri")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                ApplicationError::Internal("OIDC discovery document missing jwks_uri".to_string())
            })?
            .to_string();

        // Cache the result
        let mut cache = self.cache.write().await;
        cache.insert(
            discovery_url.to_string(),
            CachedDiscovery {
                jwks_uri: jwks_uri.clone(),
                expires_at: Instant::now() + self.ttl,
            },
        );

        Ok(jwks_uri)
    }
}
