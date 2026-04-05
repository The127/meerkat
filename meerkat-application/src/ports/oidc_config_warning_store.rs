use chrono::{DateTime, Utc};

use meerkat_domain::models::oidc_config::OidcConfigId;

use crate::error::ApplicationError;

#[derive(Debug, Clone)]
pub struct OidcConfigWarningReadModel {
    pub oidc_config_id: OidcConfigId,
    pub warning_key: String,
    pub message: String,
    pub context: Option<serde_json::Value>,
    pub first_seen: DateTime<Utc>,
    pub last_seen: DateTime<Utc>,
    pub occurrence_count: i64,
}

#[async_trait::async_trait]
#[cfg_attr(any(test, feature = "test-utils"), mockall::automock)]
pub trait OidcConfigWarningStore: Send + Sync {
    async fn upsert(
        &self,
        oidc_config_id: &OidcConfigId,
        warning_key: &str,
        message: &str,
        context: Option<&serde_json::Value>,
    ) -> Result<(), ApplicationError>;

    async fn list_by_config(
        &self,
        oidc_config_id: &OidcConfigId,
    ) -> Result<Vec<OidcConfigWarningReadModel>, ApplicationError>;

    async fn dismiss(
        &self,
        oidc_config_id: &OidcConfigId,
        warning_key: &str,
    ) -> Result<(), ApplicationError>;
}
