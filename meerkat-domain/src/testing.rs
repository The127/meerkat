use chrono::Utc;
use vec1::vec1;
use crate::models::event::{Event, EventLevel};
use crate::models::issue::{Issue, IssueId};
use crate::models::oidc_config::{Audience, ClaimMapping, ClientId, OidcConfig, RoleValues, Url};
use crate::models::organization::{Organization, OrganizationSlug};
use crate::models::project::{Project, ProjectId, ProjectSlug};
use crate::models::project_key::ProjectKey;

pub fn test_role_values() -> RoleValues {
    RoleValues::new(
        vec1!["owner".to_string()],
        vec1!["admin".to_string()],
        vec1!["member".to_string()],
    )
}

pub fn test_claim_mapping() -> ClaimMapping {
    ClaimMapping::new(
        "sub", "preferred_username", "roles",
        test_role_values(),
    ).unwrap()
}

pub fn draft_config(name: &str) -> OidcConfig {
    OidcConfig::new(
        name.into(),
        ClientId::new("meerkat-client").unwrap(),
        Url::new("https://auth.example.com").unwrap(),
        Audience::new("meerkat-api").unwrap(),
        None,
        test_claim_mapping(),
    ).unwrap()
}

pub fn test_config() -> OidcConfig {
    draft_config("Default SSO")
}

pub fn test_org() -> Organization {
    let config = draft_config("Default SSO");
    let slug = OrganizationSlug::new("test-org").unwrap();
    Organization::new("Test Org".into(), slug, config).unwrap()
}

pub fn test_project() -> Project {
    let org_id = crate::models::organization::OrganizationId::new();
    let slug = ProjectSlug::new("test-project").unwrap();
    Project::new(org_id, "Test Project".into(), slug).unwrap()
}

pub fn test_project_key() -> ProjectKey {
    let project_id = ProjectId::new();
    ProjectKey::generate(project_id, "Default".into()).unwrap()
}

pub fn test_event() -> Event {
    Event::new(
        ProjectId::new(),
        IssueId::new(),
        "abc123def456".into(),
        "Test error".into(),
        EventLevel::Error,
        "python".into(),
        Utc::now(),
        None,
        Some("production".into()),
        None,
        Some("RuntimeError".into()),
        Some("something went wrong".into()),
        vec![],
        serde_json::Value::Null,
    )
    .unwrap()
}

pub fn test_issue() -> Issue {
    Issue::new(
        "RuntimeError: something went wrong".into(),
        "abc123def456".into(),
        ProjectId::new(),
        EventLevel::Error,
        Utc::now(),
    )
    .unwrap()
}
