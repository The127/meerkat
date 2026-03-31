use chrono::{DateTime, Utc};
use meerkat_macros::{uuid_id, Reconstitute};
use crate::ports::clock::Clock;

uuid_id!(OidcConfigId);

#[derive(Debug, Clone, PartialEq, Eq, strum::Display, strum::EnumString, strum::AsRefStr)]
pub enum OidcConfigStatus {
    #[strum(serialize = "draft")]
    Draft,
    #[strum(serialize = "active")]
    Active,
    #[strum(serialize = "inactive")]
    Inactive,
}

#[derive(Debug, Clone, Reconstitute)]
pub struct OidcConfig {
    id: OidcConfigId,
    name: String,
    client_id: String,
    issuer_url: String,
    audience: String,
    jwks_url: String,
    status: OidcConfigStatus,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

#[derive(Debug, thiserror::Error)]
pub enum OidcConfigError {
    #[error("OIDC config name must not be empty")]
    EmptyName,
    #[error("OIDC issuer URL must not be empty")]
    EmptyIssuerUrl,
    #[error("OIDC audience must not be empty")]
    EmptyAudience,
    #[error("OIDC client ID must not be empty")]
    EmptyClientId,
}

impl OidcConfig {
    pub(crate) fn new_active(
        name: String,
        client_id: String,
        issuer_url: String,
        audience: String,
        jwks_url: Option<String>,
        clock: &dyn Clock,
    ) -> Result<Self, OidcConfigError> {
        Self::create(name, client_id, issuer_url, audience, jwks_url, OidcConfigStatus::Active, clock)
    }

    pub(crate) fn new_draft(
        name: String,
        client_id: String,
        issuer_url: String,
        audience: String,
        jwks_url: Option<String>,
        clock: &dyn Clock,
    ) -> Result<Self, OidcConfigError> {
        Self::create(name, client_id, issuer_url, audience, jwks_url, OidcConfigStatus::Draft, clock)
    }

    fn create(
        name: String,
        client_id: String,
        issuer_url: String,
        audience: String,
        jwks_url: Option<String>,
        status: OidcConfigStatus,
        clock: &dyn Clock,
    ) -> Result<Self, OidcConfigError> {
        let name = name.trim().to_string();
        if name.is_empty() {
            return Err(OidcConfigError::EmptyName);
        }

        let client_id = client_id.trim().to_string();
        if client_id.is_empty() {
            return Err(OidcConfigError::EmptyClientId);
        }

        let issuer_url = issuer_url.trim().to_string();
        if issuer_url.is_empty() {
            return Err(OidcConfigError::EmptyIssuerUrl);
        }

        let audience = audience.trim().to_string();
        if audience.is_empty() {
            return Err(OidcConfigError::EmptyAudience);
        }

        let jwks_url = jwks_url
            .map(|u| u.trim().to_string())
            .filter(|u| !u.is_empty())
            .unwrap_or_else(|| derive_jwks_url(&issuer_url));

        let now = clock.now();

        Ok(Self {
            id: OidcConfigId::new(),
            name,
            client_id,
            issuer_url,
            audience,
            jwks_url,
            status,
            created_at: now,
            updated_at: now,
        })
    }

    pub fn id(&self) -> &OidcConfigId { &self.id }
    pub fn name(&self) -> &str { &self.name }
    pub fn client_id(&self) -> &str { &self.client_id }
    pub fn issuer_url(&self) -> &str { &self.issuer_url }
    pub fn audience(&self) -> &str { &self.audience }
    pub fn jwks_url(&self) -> &str { &self.jwks_url }
    pub fn status(&self) -> &OidcConfigStatus { &self.status }
    pub fn created_at(&self) -> &DateTime<Utc> { &self.created_at }
    pub fn updated_at(&self) -> &DateTime<Utc> { &self.updated_at }

    pub fn is_active(&self) -> bool {
        self.status == OidcConfigStatus::Active
    }

    pub(crate) fn activate(&mut self, clock: &dyn Clock) {
        self.status = OidcConfigStatus::Active;
        self.updated_at = clock.now();
    }

    pub(crate) fn deactivate(&mut self, clock: &dyn Clock) {
        self.status = OidcConfigStatus::Inactive;
        self.updated_at = clock.now();
    }
}

fn derive_jwks_url(issuer_url: &str) -> String {
    let base = issuer_url.trim_end_matches('/');
    format!("{}/.well-known/jwks.json", base)
}

impl Default for OidcConfigStatus {
    fn default() -> Self {
        OidcConfigStatus::Draft
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ports::clock::MockClock;

    #[test]
    fn jwks_url_derived_from_issuer_when_not_provided() {
        let clock = MockClock::new(Utc::now());
        let config = OidcConfig::new_active(
            "Test".into(),
            "test-client".into(),
            "https://auth.example.com".into(),
            "my-app".into(),
            None,
            &clock,
        ).unwrap();

        assert_eq!(config.jwks_url(), "https://auth.example.com/.well-known/jwks.json");
    }

    #[test]
    fn jwks_url_derived_strips_trailing_slash() {
        let clock = MockClock::new(Utc::now());
        let config = OidcConfig::new_active(
            "Test".into(),
            "test-client".into(),
            "https://auth.example.com/".into(),
            "my-app".into(),
            None,
            &clock,
        ).unwrap();

        assert_eq!(config.jwks_url(), "https://auth.example.com/.well-known/jwks.json");
    }

    #[test]
    fn explicit_jwks_url_used_when_provided() {
        let clock = MockClock::new(Utc::now());
        let config = OidcConfig::new_active(
            "Test".into(),
            "test-client".into(),
            "https://auth.example.com".into(),
            "my-app".into(),
            Some("https://custom.example.com/jwks".into()),
            &clock,
        ).unwrap();

        assert_eq!(config.jwks_url(), "https://custom.example.com/jwks");
    }

    #[test]
    fn blank_jwks_url_falls_back_to_derived() {
        let clock = MockClock::new(Utc::now());
        let config = OidcConfig::new_active(
            "Test".into(),
            "test-client".into(),
            "https://auth.example.com".into(),
            "my-app".into(),
            Some("  ".into()),
            &clock,
        ).unwrap();

        assert_eq!(config.jwks_url(), "https://auth.example.com/.well-known/jwks.json");
    }

    #[test]
    fn empty_name_rejected() {
        let clock = MockClock::new(Utc::now());
        let result = OidcConfig::new_active("  ".into(), "cid".into(), "https://x.com".into(), "aud".into(), None, &clock);
        assert!(matches!(result, Err(OidcConfigError::EmptyName)));
    }

    #[test]
    fn empty_client_id_rejected() {
        let clock = MockClock::new(Utc::now());
        let result = OidcConfig::new_active("Name".into(), "  ".into(), "https://x.com".into(), "aud".into(), None, &clock);
        assert!(matches!(result, Err(OidcConfigError::EmptyClientId)));
    }

    #[test]
    fn empty_issuer_url_rejected() {
        let clock = MockClock::new(Utc::now());
        let result = OidcConfig::new_active("Name".into(), "cid".into(), "  ".into(), "aud".into(), None, &clock);
        assert!(matches!(result, Err(OidcConfigError::EmptyIssuerUrl)));
    }

    #[test]
    fn empty_audience_rejected() {
        let clock = MockClock::new(Utc::now());
        let result = OidcConfig::new_active("Name".into(), "cid".into(), "https://x.com".into(), "  ".into(), None, &clock);
        assert!(matches!(result, Err(OidcConfigError::EmptyAudience)));
    }

    #[test]
    fn new_active_creates_with_active_status() {
        let clock = MockClock::new(Utc::now());
        let config = OidcConfig::new_active("SSO".into(), "cid".into(), "https://x.com".into(), "aud".into(), None, &clock).unwrap();
        assert_eq!(config.status(), &OidcConfigStatus::Active);
        assert!(config.is_active());
    }

    #[test]
    fn new_draft_creates_with_draft_status() {
        let clock = MockClock::new(Utc::now());
        let config = OidcConfig::new_draft("SSO".into(), "cid".into(), "https://x.com".into(), "aud".into(), None, &clock).unwrap();
        assert_eq!(config.status(), &OidcConfigStatus::Draft);
        assert!(!config.is_active());
    }

    #[test]
    fn status_round_trips_through_string() {
        for status in [OidcConfigStatus::Draft, OidcConfigStatus::Active, OidcConfigStatus::Inactive] {
            let s = status.to_string();
            let parsed: OidcConfigStatus = s.parse().unwrap();
            assert_eq!(parsed, status);
        }
    }
}
