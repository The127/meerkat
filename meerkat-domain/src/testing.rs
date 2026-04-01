use crate::models::oidc_config::{Audience, ClientId, OidcConfig, Url};
use crate::models::organization::{Organization, OrganizationSlug};
use crate::models::project::{Project, ProjectSlug};
use crate::ports::clock::MockClock;

pub fn draft_config(name: &str, clock: &MockClock) -> OidcConfig {
    OidcConfig::new(
        name.into(),
        ClientId::new("meerkat-client").unwrap(),
        Url::new("https://auth.example.com").unwrap(),
        Audience::new("meerkat-api").unwrap(),
        None, clock,
    ).unwrap()
}

pub fn test_config() -> (OidcConfig, MockClock) {
    let clock = MockClock::new(chrono::Utc::now());
    let config = draft_config("Default SSO", &clock);
    (config, clock)
}

pub fn test_org() -> (Organization, MockClock) {
    let clock = MockClock::new(chrono::Utc::now());
    let config = draft_config("Default SSO", &clock);
    let slug = OrganizationSlug::new("test-org").unwrap();
    let org = Organization::new("Test Org".into(), slug, config, &clock).unwrap();
    (org, clock)
}

pub fn test_project() -> (Project, MockClock) {
    let clock = MockClock::new(chrono::Utc::now());
    let org_id = crate::models::organization::OrganizationId::new();
    let slug = ProjectSlug::new("test-project").unwrap();
    let project = Project::new(org_id, "Test Project".into(), slug, &clock).unwrap();
    (project, clock)
}
