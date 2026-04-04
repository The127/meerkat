use meerkat_macros::{uuid_id, Reconstitute};
use vec1::Vec1;
use crate::models::org_role::OrgRole;
pub use crate::shared::url::Url;

uuid_id!(OidcConfigId);

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "openapi", schema(value_type = String))]
#[serde(transparent)]
pub struct ClientId(String);

impl<'de> serde::Deserialize<'de> for ClientId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Self::new(s).map_err(serde::de::Error::custom)
    }
}

impl ClientId {
    pub fn new(value: impl Into<String>) -> Result<Self, OidcConfigError> {
        let value = value.into().trim().to_string();
        if value.is_empty() {
            return Err(OidcConfigError::EmptyClientId);
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str { &self.0 }
}

impl std::fmt::Display for ClientId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "openapi", schema(value_type = String))]
#[serde(transparent)]
pub struct Audience(String);

impl<'de> serde::Deserialize<'de> for Audience {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Self::new(s).map_err(serde::de::Error::custom)
    }
}

impl Audience {
    pub fn new(value: impl Into<String>) -> Result<Self, OidcConfigError> {
        let value = value.into().trim().to_string();
        if value.is_empty() {
            return Err(OidcConfigError::EmptyAudience);
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str { &self.0 }
}

impl std::fmt::Display for Audience {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClaimName(String);

impl ClaimName {
    pub fn new(value: impl Into<String>) -> Result<Self, OidcConfigError> {
        let value = value.into().trim().to_string();
        if value.is_empty() {
            return Err(OidcConfigError::EmptyClaimName);
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str { &self.0 }
}

impl std::fmt::Display for ClaimName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoleValues {
    owner: Vec1<String>,
    admin: Vec1<String>,
    member: Vec1<String>,
}

impl RoleValues {
    pub fn new(
        owner: Vec1<String>,
        admin: Vec1<String>,
        member: Vec1<String>,
    ) -> Self {
        Self { owner, admin, member }
    }

    pub fn resolve_roles(&self, claim_values: &[&str]) -> Vec<OrgRole> {
        let mut roles = Vec::new();

        let has_owner = claim_values.iter().any(|v| self.owner.iter().any(|o| o == v));
        let has_admin = claim_values.iter().any(|v| self.admin.iter().any(|a| a == v));
        let has_member = claim_values.iter().any(|v| self.member.iter().any(|m| m == v));

        if has_owner { roles.push(OrgRole::Owner); }
        if has_admin { roles.push(OrgRole::Admin); }
        if has_member { roles.push(OrgRole::Member); }

        roles
    }

    pub fn owner(&self) -> &Vec1<String> { &self.owner }
    pub fn admin(&self) -> &Vec1<String> { &self.admin }
    pub fn member(&self) -> &Vec1<String> { &self.member }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClaimMapping {
    sub_claim: ClaimName,
    name_claim: ClaimName,
    role_claim: ClaimName,
    role_values: RoleValues,
}

impl ClaimMapping {
    pub fn new(
        sub_claim: impl Into<String>,
        name_claim: impl Into<String>,
        role_claim: impl Into<String>,
        role_values: RoleValues,
    ) -> Result<Self, OidcConfigError> {
        let sub_claim = ClaimName::new(sub_claim)?;
        let name_claim = ClaimName::new(name_claim)?;
        let role_claim = ClaimName::new(role_claim)?;

        Ok(Self {
            sub_claim,
            name_claim,
            role_claim,
            role_values,
        })
    }

    pub fn resolve_roles(&self, claim_values: &[&str]) -> Vec<OrgRole> {
        self.role_values.resolve_roles(claim_values)
    }

    pub fn sub_claim(&self) -> &ClaimName { &self.sub_claim }
    pub fn name_claim(&self) -> &ClaimName { &self.name_claim }
    pub fn role_claim(&self) -> &ClaimName { &self.role_claim }
    pub fn role_values(&self) -> &RoleValues { &self.role_values }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, strum::Display, strum::EnumString, strum::AsRefStr)]
pub enum OidcConfigStatus {
    #[strum(serialize = "draft")]
    #[default]
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
    client_id: ClientId,
    issuer_url: Url,
    audience: Audience,
    discovery_url: Option<Url>,
    claim_mapping: ClaimMapping,
    status: OidcConfigStatus,
}

#[derive(Debug, thiserror::Error)]
pub enum OidcConfigError {
    #[error("OIDC config name must not be empty")]
    EmptyName,
    #[error("OIDC client ID must not be empty")]
    EmptyClientId,
    #[error("OIDC audience must not be empty")]
    EmptyAudience,
    #[error("claim name must not be empty")]
    EmptyClaimName,
    #[error("invalid OIDC config status transition from {from} to {to}")]
    InvalidStatusTransition { from: OidcConfigStatus, to: OidcConfigStatus },
}

impl OidcConfig {
    pub fn new(
        name: String,
        client_id: ClientId,
        issuer_url: Url,
        audience: Audience,
        discovery_url: Option<Url>,
        claim_mapping: ClaimMapping,
    ) -> Result<Self, OidcConfigError> {
        let name = name.trim().to_string();
        if name.is_empty() {
            return Err(OidcConfigError::EmptyName);
        }

        let id = OidcConfigId::new();

        Ok(Self {
            id,
            name,
            client_id,
            issuer_url,
            audience,
            discovery_url,
            claim_mapping,
            status: OidcConfigStatus::Draft,
        })
    }

    pub fn id(&self) -> &OidcConfigId { &self.id }
    pub fn name(&self) -> &str { &self.name }
    pub fn client_id(&self) -> &ClientId { &self.client_id }
    pub fn issuer_url(&self) -> &Url { &self.issuer_url }
    pub fn audience(&self) -> &Audience { &self.audience }
    pub fn discovery_url(&self) -> Option<&Url> { self.discovery_url.as_ref() }
    pub fn claim_mapping(&self) -> &ClaimMapping { &self.claim_mapping }
    pub fn status(&self) -> &OidcConfigStatus { &self.status }

    pub fn is_active(&self) -> bool {
        self.status == OidcConfigStatus::Active
    }

    pub fn activate(&mut self) -> Result<(), OidcConfigError> {
        match self.status {
            OidcConfigStatus::Draft | OidcConfigStatus::Inactive => {
                self.status = OidcConfigStatus::Active;
                Ok(())
            }
            OidcConfigStatus::Active => Err(OidcConfigError::InvalidStatusTransition {
                from: OidcConfigStatus::Active,
                to: OidcConfigStatus::Active,
            }),
        }
    }

    pub fn deactivate(&mut self) -> Result<(), OidcConfigError> {
        match self.status {
            OidcConfigStatus::Active => {
                self.status = OidcConfigStatus::Inactive;
                Ok(())
            }
            OidcConfigStatus::Inactive => Err(OidcConfigError::InvalidStatusTransition {
                from: OidcConfigStatus::Inactive,
                to: OidcConfigStatus::Inactive,
            }),
            OidcConfigStatus::Draft => Err(OidcConfigError::InvalidStatusTransition {
                from: OidcConfigStatus::Draft,
                to: OidcConfigStatus::Inactive,
            }),
        }
    }

    pub fn update_claim_mapping(&mut self, claim_mapping: ClaimMapping) {
        if self.claim_mapping == claim_mapping {
            return;
        }

        self.claim_mapping = claim_mapping;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use vec1::vec1;
    use crate::models::org_role::OrgRole;
    use crate::testing::{test_claim_mapping, test_config, test_role_values};

    // --- creation ---

    #[test]
    fn given_valid_input_then_creation_succeeds_with_draft_status() {
        // act
        let config = OidcConfig::new(
            "My SSO".into(),
            ClientId::new("client-id").unwrap(),
            Url::new("https://auth.example.com").unwrap(),
            Audience::new("my-api").unwrap(),
            Some(Url::new("https://auth.example.com/.well-known/openid-configuration").unwrap()),
            test_claim_mapping(),
        ).unwrap();

        // assert
        assert_eq!(config.name(), "My SSO");
        assert_eq!(config.client_id().as_str(), "client-id");
        assert_eq!(config.issuer_url().as_str(), "https://auth.example.com");
        assert_eq!(config.audience().as_str(), "my-api");
        assert_eq!(config.discovery_url().unwrap().as_str(), "https://auth.example.com/.well-known/openid-configuration");
        assert_eq!(config.claim_mapping().sub_claim().as_str(), "sub");
        assert_eq!(config.status(), &OidcConfigStatus::Draft);
        assert!(!config.is_active());
    }

    #[test]
    fn given_no_discovery_url_then_discovery_url_is_none() {
        // arrange / act
        let config = test_config();

        // assert
        assert!(config.discovery_url().is_none());
    }

    #[test]
    fn given_empty_name_then_creation_fails() {
        // act
        let result = OidcConfig::new(
            "  ".into(),
            ClientId::new("cid").unwrap(),
            Url::new("https://x.com").unwrap(),
            Audience::new("aud").unwrap(),
            None, test_claim_mapping(),
        );

        // assert
        match result {
            Err(OidcConfigError::EmptyName) => (),
            _ => panic!("Expected EmptyName error, got {:?}", result),
        }
    }

    #[test]
    fn given_empty_client_id_then_creation_fails() {
        // act
        let result = ClientId::new("  ");

        // assert
        match result {
            Err(OidcConfigError::EmptyClientId) => (),
            _ => panic!("Expected EmptyClientId error, got {:?}", result),
        }
    }

    #[test]
    fn given_empty_audience_then_creation_fails() {
        // act
        let result = Audience::new("  ");

        // assert
        match result {
            Err(OidcConfigError::EmptyAudience) => (),
            _ => panic!("Expected EmptyAudience error, got {:?}", result),
        }
    }

    // --- claim mapping ---

    #[test]
    fn given_empty_claim_name_then_creation_fails() {
        // act
        let result = ClaimName::new("  ");

        // assert
        match result {
            Err(OidcConfigError::EmptyClaimName) => (),
            _ => panic!("Expected EmptyClaimName error, got {:?}", result),
        }
    }

    #[test]
    fn given_empty_claim_name_in_mapping_then_creation_fails() {
        // act
        let result = ClaimMapping::new("", "name", "roles", test_role_values());

        // assert
        match result {
            Err(OidcConfigError::EmptyClaimName) => (),
            _ => panic!("Expected EmptyClaimName error, got {:?}", result),
        }
    }

    #[test]
    fn given_valid_claim_mapping_then_creation_succeeds() {
        // act
        let mapping = ClaimMapping::new(
            "sub", "preferred_username", "roles",
            test_role_values(),
        ).unwrap();

        // assert
        assert_eq!(mapping.sub_claim().as_str(), "sub");
        assert_eq!(mapping.name_claim().as_str(), "preferred_username");
        assert_eq!(mapping.role_claim().as_str(), "roles");
    }

    #[test]
    fn given_owner_claim_value_then_resolve_roles_returns_owner() {
        // arrange
        let mapping = test_claim_mapping();

        // act
        let roles = mapping.resolve_roles(&["owner"]);

        // assert
        assert_eq!(roles, vec![OrgRole::Owner]);
    }

    #[test]
    fn given_multiple_matching_values_then_resolve_roles_returns_all_matching_roles() {
        // arrange
        let mapping = test_claim_mapping();

        // act
        let roles = mapping.resolve_roles(&["admin", "member"]);

        // assert
        assert_eq!(roles, vec![OrgRole::Admin, OrgRole::Member]);
    }

    #[test]
    fn given_no_matching_values_then_resolve_roles_returns_empty() {
        // arrange
        let mapping = test_claim_mapping();

        // act
        let roles = mapping.resolve_roles(&["viewer"]);

        // assert
        assert!(roles.is_empty());
    }

    #[test]
    fn given_config_then_update_claim_mapping_updates() {
        // arrange
        let mut config = test_config();
        let new_mapping = ClaimMapping::new(
            "sub", "name", "groups",
            RoleValues::new(vec1!["superadmin".into()], vec1!["staff".into()], vec1!["user".into()]),
        ).unwrap();

        // act
        config.update_claim_mapping(new_mapping);

        // assert
        assert_eq!(config.claim_mapping().role_claim().as_str(), "groups");
    }

    #[test]
    fn given_same_claim_mapping_then_update_is_idempotent() {
        // arrange
        let mut config = test_config();
        let original = config.claim_mapping().clone();

        // act
        config.update_claim_mapping(test_claim_mapping());

        // assert
        assert_eq!(config.claim_mapping(), &original);
    }

    // --- activate ---

    #[test]
    fn given_draft_config_then_activate_transitions_to_active() {
        // arrange
        let mut config = test_config();

        // act
        config.activate().unwrap();

        // assert
        assert_eq!(config.status(), &OidcConfigStatus::Active);
        assert!(config.is_active());
    }

    #[test]
    fn given_inactive_config_then_activate_transitions_to_active() {
        // arrange
        let mut config = test_config();
        config.activate().unwrap();
        config.deactivate().unwrap();

        // act
        config.activate().unwrap();

        // assert
        assert_eq!(config.status(), &OidcConfigStatus::Active);
    }

    #[test]
    fn given_active_config_then_activate_fails() {
        // arrange
        let mut config = test_config();
        config.activate().unwrap();

        // act
        let result = config.activate();

        // assert
        match result {
            Err(OidcConfigError::InvalidStatusTransition { from, to }) => {
                assert_eq!(from, OidcConfigStatus::Active);
                assert_eq!(to, OidcConfigStatus::Active);
            },
            _ => panic!("Expected InvalidStatusTransition error, got {:?}", result),
        }
    }

    // --- deactivate ---

    #[test]
    fn given_active_config_then_deactivate_transitions_to_inactive() {
        // arrange
        let mut config = test_config();
        config.activate().unwrap();

        // act
        config.deactivate().unwrap();

        // assert
        assert_eq!(config.status(), &OidcConfigStatus::Inactive);
        assert!(!config.is_active());
    }

    #[test]
    fn given_draft_config_then_deactivate_fails() {
        // arrange
        let mut config = test_config();

        // act
        let result = config.deactivate();

        // assert
        match result {
            Err(OidcConfigError::InvalidStatusTransition { from, to }) => {
                assert_eq!(from, OidcConfigStatus::Draft);
                assert_eq!(to, OidcConfigStatus::Inactive);
            },
            _ => panic!("Expected InvalidStatusTransition error, got {:?}", result),
        }
    }

    #[test]
    fn given_inactive_config_then_deactivate_fails() {
        // arrange
        let mut config = test_config();
        config.activate().unwrap();
        config.deactivate().unwrap();

        // act
        let result = config.deactivate();

        // assert
        match result {
            Err(OidcConfigError::InvalidStatusTransition { from, to }) => {
                assert_eq!(from, OidcConfigStatus::Inactive);
                assert_eq!(to, OidcConfigStatus::Inactive);
            },
            _ => panic!("Expected InvalidStatusTransition error, got {:?}", result),
        }
    }
}
