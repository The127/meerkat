use meerkat_macros::{uuid_id, Reconstitute};
use crate::shared::version::Version;
use crate::models::project::ProjectId;

uuid_id!(ProjectKeyId);

#[derive(Debug, Clone, PartialEq, Eq, strum::Display, strum::EnumString, strum::AsRefStr)]
pub enum ProjectKeyStatus {
    #[strum(serialize = "active")]
    Active,
    #[strum(serialize = "revoked")]
    Revoked,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RateLimit(u64);

impl RateLimit {
    pub fn new(value: u64) -> Result<Self, ProjectKeyError> {
        if value == 0 {
            return Err(ProjectKeyError::ZeroRateLimit);
        }
        Ok(Self(value))
    }

    pub fn value(&self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(transparent)]
pub struct KeyToken(String);

impl KeyToken {
    pub fn generate() -> Self {
        let bytes: [u8; 16] = rand::random();
        let hex = bytes.iter().map(|b| format!("{:02x}", b)).collect::<String>();
        Self(hex)
    }

    pub fn new(value: impl Into<String>) -> Result<Self, ProjectKeyError> {
        let value = value.into();
        if value.is_empty() {
            return Err(ProjectKeyError::EmptyKeyToken);
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, Reconstitute)]
pub struct ProjectKey {
    id: ProjectKeyId,
    project_id: ProjectId,
    key_token: KeyToken,
    label: String,
    status: ProjectKeyStatus,
    rate_limit: Option<RateLimit>,
    version: Version,
}

#[derive(Debug, thiserror::Error)]
pub enum ProjectKeyError {
    #[error("project key label must not be empty")]
    EmptyLabel,
    #[error("key token must not be empty")]
    EmptyKeyToken,
    #[error("project key is already revoked")]
    AlreadyRevoked,
    #[error("rate limit must be greater than zero")]
    ZeroRateLimit,
}

impl ProjectKey {
    pub fn generate(
        project_id: ProjectId,
        label: String,
    ) -> Result<Self, ProjectKeyError> {
        let label = label.trim().to_string();
        if label.is_empty() {
            return Err(ProjectKeyError::EmptyLabel);
        }

        let id = ProjectKeyId::new();
        let key_token = KeyToken::generate();

        Ok(ProjectKey {
            id,
            project_id,
            key_token,
            label,
            status: ProjectKeyStatus::Active,
            rate_limit: None,
            version: Version::initial(),
        })
    }

    pub fn revoke(&mut self) -> Result<(), ProjectKeyError> {
        if self.status == ProjectKeyStatus::Revoked {
            return Err(ProjectKeyError::AlreadyRevoked);
        }

        self.status = ProjectKeyStatus::Revoked;
        Ok(())
    }

    pub fn set_rate_limit(&mut self, limit: Option<RateLimit>) {
        self.rate_limit = limit;
    }

    pub fn id(&self) -> &ProjectKeyId { &self.id }
    pub fn project_id(&self) -> &ProjectId { &self.project_id }
    pub fn key_token(&self) -> &KeyToken { &self.key_token }
    pub fn label(&self) -> &str { &self.label }
    pub fn status(&self) -> &ProjectKeyStatus { &self.status }
    pub fn rate_limit(&self) -> Option<RateLimit> { self.rate_limit }
    pub fn version(&self) -> &Version { &self.version }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::test_project_key;

    #[test]
    fn given_valid_input_then_generate_succeeds() {
        // arrange
        let project_id = ProjectId::new();

        // act
        let key = ProjectKey::generate(project_id.clone(), "Default".into())
            .expect("Failed to generate key");

        // assert
        assert_eq!(key.project_id(), &project_id);
        assert_eq!(key.label(), "Default");
        assert_eq!(key.status(), &ProjectKeyStatus::Active);
        assert_eq!(key.key_token().as_str().len(), 32);
    }

    #[test]
    fn given_empty_label_then_generate_fails() {
        // act
        let result = ProjectKey::generate(ProjectId::new(), "  ".into());

        // assert
        match result {
            Err(ProjectKeyError::EmptyLabel) => (),
            other => panic!("Expected EmptyLabel error, got {:?}", other),
        }
    }

    #[test]
    fn given_active_key_then_revoke_succeeds() {
        // arrange
        let mut key = test_project_key();

        // act
        key.revoke().expect("Failed to revoke key");

        // assert
        assert_eq!(key.status(), &ProjectKeyStatus::Revoked);
    }

    #[test]
    fn given_revoked_key_then_revoke_fails() {
        // arrange
        let mut key = test_project_key();
        key.revoke().unwrap();

        // act
        let result = key.revoke();

        // assert
        match result {
            Err(ProjectKeyError::AlreadyRevoked) => (),
            other => panic!("Expected AlreadyRevoked error, got {:?}", other),
        }
    }

    #[test]
    fn given_two_generated_keys_then_key_tokens_are_unique() {
        // arrange
        let project_id = ProjectId::new();

        // act
        let key1 = ProjectKey::generate(project_id.clone(), "Key 1".into()).unwrap();
        let key2 = ProjectKey::generate(project_id, "Key 2".into()).unwrap();

        // assert
        assert_ne!(key1.key_token().as_str(), key2.key_token().as_str());
    }

    #[test]
    fn given_zero_rate_limit_then_creation_fails() {
        // act
        let result = RateLimit::new(0);

        // assert
        match result {
            Err(ProjectKeyError::ZeroRateLimit) => (),
            other => panic!("Expected ZeroRateLimit error, got {:?}", other),
        }
    }

    #[test]
    fn given_valid_rate_limit_then_set_succeeds() {
        // arrange
        let mut key = test_project_key();
        let limit = RateLimit::new(500).unwrap();

        // act
        key.set_rate_limit(Some(limit));

        // assert
        assert_eq!(key.rate_limit().unwrap().value(), 500);
    }

    #[test]
    fn given_label_with_whitespace_then_label_is_trimmed() {
        // act
        let key = ProjectKey::generate(ProjectId::new(), "  My Key  ".into()).unwrap();

        // assert
        assert_eq!(key.label(), "My Key");
    }
}
