use std::collections::HashMap;
use std::time::{Duration, Instant};

use async_trait::async_trait;
use tokio::sync::RwLock;

use meerkat_application::error::ApplicationError;
use meerkat_application::ports::jwks_provider::JwksProvider;

struct CachedEntry {
    keys: Vec<serde_json::Value>,
    expires_at: Instant,
}

pub struct CachedJwksProvider {
    client: reqwest::Client,
    cache: RwLock<HashMap<String, CachedEntry>>,
    ttl: Duration,
}

impl CachedJwksProvider {
    pub fn new(ttl: Duration) -> Self {
        Self {
            client: reqwest::Client::new(),
            cache: RwLock::new(HashMap::new()),
            ttl,
        }
    }

    async fn fetch_remote(&self, url: &str) -> Result<Vec<serde_json::Value>, ApplicationError> {
        let json = self
            .client
            .get(url)
            .send()
            .await
            .map_err(|e| ApplicationError::Internal(format!("JWKS fetch failed: {e}")))?
            .error_for_status()
            .map_err(|e| ApplicationError::Internal(format!("JWKS fetch failed: {e}")))?
            .text()
            .await
            .map_err(|e| ApplicationError::Internal(format!("JWKS read failed: {e}")))?;

        let keys = parse_jwks_keys(&json)?;

        let mut cache = self.cache.write().await;
        cache.insert(
            url.to_string(),
            CachedEntry {
                keys: keys.clone(),
                expires_at: Instant::now() + self.ttl,
            },
        );

        Ok(keys)
    }

    async fn get_cached(&self, url: &str) -> Option<Vec<serde_json::Value>> {
        let cache = self.cache.read().await;
        cache
            .get(url)
            .filter(|entry| entry.expires_at > Instant::now())
            .map(|entry| entry.keys.clone())
    }
}

fn parse_jwks_keys(json: &str) -> Result<Vec<serde_json::Value>, ApplicationError> {
    let jwks: serde_json::Value = serde_json::from_str(json)
        .map_err(|e| ApplicationError::Internal(format!("invalid JWKS JSON: {e}")))?;

    let keys = jwks
        .get("keys")
        .and_then(|k| k.as_array())
        .ok_or_else(|| ApplicationError::Internal("JWKS missing keys array".to_string()))?;

    Ok(keys.clone())
}

fn find_jwk(keys: &[serde_json::Value], kid: Option<&str>) -> Option<serde_json::Value> {
    match kid {
        Some(kid) => keys.iter().find(|k| {
            k.get("kid").and_then(|v| v.as_str()) == Some(kid)
        }).cloned(),
        None => keys.first().cloned(),
    }
}

#[async_trait]
impl JwksProvider for CachedJwksProvider {
    async fn resolve_jwk(
        &self,
        jwks_url: &str,
        kid: Option<&str>,
    ) -> Result<serde_json::Value, ApplicationError> {
        // Try cached JWKS first
        if let Some(keys) = self.get_cached(jwks_url).await
            && let Some(jwk) = find_jwk(&keys, kid)
        {
            return Ok(jwk);
        }

        // Cache miss or kid not found — fetch fresh
        let keys = self.fetch_remote(jwks_url).await?;
        find_jwk(&keys, kid).ok_or(ApplicationError::NotFound)
    }
}
