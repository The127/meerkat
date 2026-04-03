use vec1::vec1;
use crate::models::oidc_config::{Audience, ClaimMapping, ClientId, OidcConfig, Url};
use crate::models::organization::{Organization, OrganizationSlug};
use crate::models::project::{Project, ProjectId, ProjectSlug};
use crate::models::project_key::ProjectKey;
use crate::ports::clock::MockClock;

pub fn test_claim_mapping() -> ClaimMapping {
    ClaimMapping::new(
        "sub", "preferred_username", "roles",
        vec1!["owner".to_string()],
        vec1!["admin".to_string()],
        vec1!["member".to_string()],
    ).unwrap()
}

pub fn draft_config(name: &str, clock: &MockClock) -> OidcConfig {
    OidcConfig::new(
        name.into(),
        ClientId::new("meerkat-client").unwrap(),
        Url::new("https://auth.example.com").unwrap(),
        Audience::new("meerkat-api").unwrap(),
        None,
        test_claim_mapping(),
        clock,
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

pub fn test_project_key() -> (ProjectKey, MockClock) {
    let clock = MockClock::new(chrono::Utc::now());
    let project_id = ProjectId::new();
    let key = ProjectKey::generate(project_id, "Default".into(), &clock).unwrap();
    (key, clock)
}
