use chrono::{DateTime, Utc};
use meerkat_macros::{uuid_id, Reconstitute};
use crate::shared::version::Version;
use crate::shared::change_tracker::ChangeTracker;
use crate::models::project::ProjectId;
use crate::ports::clock::Clock;

uuid_id!(ProjectKeyId);

#[derive(Debug, Clone, PartialEq, Eq, strum::Display, strum::EnumString, strum::AsRefStr)]
pub enum ProjectKeyStatus {
    #[strum(serialize = "active")]
    Active,
    #[strum(serialize = "revoked")]
    Revoked,
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
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    version: Version,
    #[reconstitute_ignore]
    changes: ChangeTracker<ProjectKeyChange>,
}

#[derive(Debug, Clone)]
pub enum ProjectKeyChange {
    Generated {
        id: ProjectKeyId,
        project_id: ProjectId,
        key_token: KeyToken,
        label: String,
    },
    Revoked {
        id: ProjectKeyId,
    },
}

#[derive(Debug, thiserror::Error)]
pub enum ProjectKeyError {
    #[error("project key label must not be empty")]
    EmptyLabel,
    #[error("key token must not be empty")]
    EmptyKeyToken,
    #[error("project key is already revoked")]
    AlreadyRevoked,
}

impl ProjectKey {
    pub fn generate(
        project_id: ProjectId,
        label: String,
        clock: &dyn Clock,
    ) -> Result<Self, ProjectKeyError> {
        let label = label.trim().to_string();
        if label.is_empty() {
            return Err(ProjectKeyError::EmptyLabel);
        }

        let id = ProjectKeyId::new();
        let key_token = KeyToken::generate();
        let now = clock.now();

        let mut changes = ChangeTracker::new();
        changes.record(ProjectKeyChange::Generated {
            id: id.clone(),
            project_id: project_id.clone(),
            key_token: key_token.clone(),
            label: label.clone(),
        });

        Ok(ProjectKey {
            id,
            project_id,
            key_token,
            label,
            status: ProjectKeyStatus::Active,
            created_at: now,
            updated_at: now,
            version: Version::initial(),
            changes,
        })
    }

    pub fn revoke(&mut self) -> Result<(), ProjectKeyError> {
        if self.status == ProjectKeyStatus::Revoked {
            return Err(ProjectKeyError::AlreadyRevoked);
        }

        self.status = ProjectKeyStatus::Revoked;
        self.changes.record(ProjectKeyChange::Revoked {
            id: self.id.clone(),
        });

        Ok(())
    }

    pub fn pull_changes(&mut self) -> Vec<ProjectKeyChange> {
        self.changes.pull_changes()
    }

    pub fn id(&self) -> &ProjectKeyId { &self.id }
    pub fn project_id(&self) -> &ProjectId { &self.project_id }
    pub fn key_token(&self) -> &KeyToken { &self.key_token }
    pub fn label(&self) -> &str { &self.label }
    pub fn status(&self) -> &ProjectKeyStatus { &self.status }
    pub fn created_at(&self) -> &DateTime<Utc> { &self.created_at }
    pub fn updated_at(&self) -> &DateTime<Utc> { &self.updated_at }
    pub fn version(&self) -> &Version { &self.version }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ports::clock::MockClock;
    use crate::testing::test_project_key;

    #[test]
    fn given_valid_input_then_generate_succeeds_and_records_change() {
        // arrange
        let project_id = ProjectId::new();
        let expected_now = Utc::now();
        let clock = MockClock::new(expected_now);

        // act
        let mut key = ProjectKey::generate(project_id.clone(), "Default".into(), &clock)
            .expect("Failed to generate key");

        // assert
        assert_eq!(key.project_id(), &project_id);
        assert_eq!(key.label(), "Default");
        assert_eq!(key.status(), &ProjectKeyStatus::Active);
        assert_eq!(key.created_at(), &expected_now);
        assert_eq!(key.key_token().as_str().len(), 32);

        let changes = key.pull_changes();
        assert_eq!(changes.len(), 1);
        match &changes[0] {
            ProjectKeyChange::Generated { id, project_id: pid, key_token, label } => {
                assert_eq!(id, key.id());
                assert_eq!(pid, &project_id);
                assert_eq!(key_token, key.key_token());
                assert_eq!(label, "Default");
            }
            _ => panic!("Expected Generated change"),
        }
    }

    #[test]
    fn given_empty_label_then_generate_fails() {
        // arrange
        let clock = MockClock::new(Utc::now());

        // act
        let result = ProjectKey::generate(ProjectId::new(), "  ".into(), &clock);

        // assert
        match result {
            Err(ProjectKeyError::EmptyLabel) => (),
            other => panic!("Expected EmptyLabel error, got {:?}", other),
        }
    }

    #[test]
    fn given_active_key_then_revoke_succeeds_and_records_change() {
        // arrange
        let (mut key, _) = test_project_key();
        let _ = key.pull_changes();

        // act
        key.revoke().expect("Failed to revoke key");

        // assert
        assert_eq!(key.status(), &ProjectKeyStatus::Revoked);

        let changes = key.pull_changes();
        assert_eq!(changes.len(), 1);
        match &changes[0] {
            ProjectKeyChange::Revoked { id } => {
                assert_eq!(id, key.id());
            }
            _ => panic!("Expected Revoked change"),
        }
    }

    #[test]
    fn given_revoked_key_then_revoke_fails() {
        // arrange
        let (mut key, _) = test_project_key();
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
        let clock = MockClock::new(Utc::now());
        let project_id = ProjectId::new();

        // act
        let key1 = ProjectKey::generate(project_id.clone(), "Key 1".into(), &clock).unwrap();
        let key2 = ProjectKey::generate(project_id, "Key 2".into(), &clock).unwrap();

        // assert
        assert_ne!(key1.key_token().as_str(), key2.key_token().as_str());
    }

    #[test]
    fn given_label_with_whitespace_then_label_is_trimmed() {
        // arrange
        let clock = MockClock::new(Utc::now());

        // act
        let key = ProjectKey::generate(ProjectId::new(), "  My Key  ".into(), &clock).unwrap();

        // assert
        assert_eq!(key.label(), "My Key");
    }
}
