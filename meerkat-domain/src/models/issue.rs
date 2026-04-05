use std::fmt;

use chrono::{DateTime, Utc};
use meerkat_macros::{uuid_id, Reconstitute};
use crate::models::event::EventLevel;
use crate::models::project::ProjectId;
use crate::shared::version::Version;

uuid_id!(IssueId);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IssueNumber(u64);

impl IssueNumber {
    pub fn new(value: u64) -> Self {
        Self(value)
    }

    pub fn value(&self) -> u64 {
        self.0
    }
}

impl fmt::Display for IssueNumber {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "#{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FingerprintHash(String);

impl FingerprintHash {
    pub fn new(value: impl Into<String>) -> Result<Self, IssueError> {
        let value = value.into().trim().to_string();
        if value.is_empty() {
            return Err(IssueError::EmptyFingerprintHash);
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone)]
pub enum IssueIdentifier {
    Id(IssueId),
    Fingerprint(ProjectId, FingerprintHash),
    Number(ProjectId, IssueNumber),
}

#[derive(Debug, Clone, Default, PartialEq, Eq, strum::Display, strum::EnumString, strum::AsRefStr)]
pub enum IssueStatus {
    #[strum(serialize = "unresolved")]
    #[default]
    Unresolved,
    #[strum(serialize = "resolved")]
    Resolved,
    #[strum(serialize = "ignored")]
    Ignored,
}

#[derive(Debug, thiserror::Error)]
pub enum IssueError {
    #[error("issue title must not be empty")]
    EmptyTitle,
    #[error("issue fingerprint hash must not be empty")]
    EmptyFingerprintHash,
    #[error("issue is already resolved")]
    AlreadyResolved,
    #[error("issue is already unresolved")]
    AlreadyUnresolved,
    #[error("issue is already ignored")]
    AlreadyIgnored,
}

#[derive(Debug, Clone, Reconstitute)]
pub struct Issue {
    id: IssueId,
    project_id: ProjectId,
    issue_number: Option<IssueNumber>,
    title: String,
    fingerprint_hash: FingerprintHash,
    status: IssueStatus,
    level: EventLevel,
    event_count: u64,
    first_seen: DateTime<Utc>,
    last_seen: DateTime<Utc>,
    version: Version,
}

impl Issue {
    pub fn derive_title(
        exception_type: Option<&str>,
        exception_value: Option<&str>,
        message: &str,
    ) -> String {
        match (exception_type, exception_value) {
            (Some(t), Some(v)) => format!("{t}: {v}"),
            (Some(t), None) => t.to_string(),
            _ => message.to_string(),
        }
    }

    pub fn new(
        title: String,
        fingerprint_hash: FingerprintHash,
        project_id: ProjectId,
        level: EventLevel,
        timestamp: DateTime<Utc>,
    ) -> Result<Self, IssueError> {
        let title = title.trim().to_string();
        if title.is_empty() {
            return Err(IssueError::EmptyTitle);
        }

        Ok(Self {
            id: IssueId::new(),
            project_id,
            issue_number: None,
            title,
            fingerprint_hash,
            status: IssueStatus::Unresolved,
            level,
            event_count: 1,
            first_seen: timestamp,
            last_seen: timestamp,
            version: Version::initial(),
        })
    }

    pub fn record_event(&mut self, level: EventLevel, timestamp: DateTime<Utc>) {
        self.event_count += 1;
        if timestamp > self.last_seen {
            self.last_seen = timestamp;
        }
        if level.severity() > self.level.severity() {
            self.level = level;
        }
    }

    pub fn resolve(&mut self) -> Result<(), IssueError> {
        if self.status == IssueStatus::Resolved {
            return Err(IssueError::AlreadyResolved);
        }
        self.status = IssueStatus::Resolved;
        Ok(())
    }

    pub fn reopen(&mut self) -> Result<(), IssueError> {
        if self.status == IssueStatus::Unresolved {
            return Err(IssueError::AlreadyUnresolved);
        }
        self.status = IssueStatus::Unresolved;
        Ok(())
    }

    pub fn ignore(&mut self) -> Result<(), IssueError> {
        if self.status == IssueStatus::Ignored {
            return Err(IssueError::AlreadyIgnored);
        }
        self.status = IssueStatus::Ignored;
        Ok(())
    }

    pub fn id(&self) -> &IssueId { &self.id }
    pub fn project_id(&self) -> &ProjectId { &self.project_id }
    pub fn issue_number(&self) -> Option<IssueNumber> { self.issue_number }
    pub fn title(&self) -> &str { &self.title }
    pub fn fingerprint_hash(&self) -> &FingerprintHash { &self.fingerprint_hash }
    pub fn status(&self) -> &IssueStatus { &self.status }
    pub fn level(&self) -> &EventLevel { &self.level }
    pub fn event_count(&self) -> u64 { self.event_count }
    pub fn first_seen(&self) -> &DateTime<Utc> { &self.first_seen }
    pub fn last_seen(&self) -> &DateTime<Utc> { &self.last_seen }
    pub fn version(&self) -> &Version { &self.version }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::test_issue;

    #[test]
    fn given_valid_input_then_creation_succeeds() {
        // arrange
        let project_id = ProjectId::new();
        let now = Utc::now();

        // act
        let issue = Issue::new(
            "TypeError: Cannot read property 'x'".into(),
            FingerprintHash::new("abc123").unwrap(),
            project_id.clone(),
            EventLevel::Error,
            now,
        )
        .expect("Failed to create issue");

        // assert
        assert_eq!(issue.project_id(), &project_id);
        assert_eq!(issue.title(), "TypeError: Cannot read property 'x'");
        assert_eq!(issue.fingerprint_hash().as_str(), "abc123");
        assert_eq!(issue.status(), &IssueStatus::Unresolved);
        assert_eq!(issue.level(), &EventLevel::Error);
        assert_eq!(issue.event_count(), 1);
        assert_eq!(issue.first_seen(), &now);
        assert_eq!(issue.last_seen(), &now);
    }

    #[test]
    fn given_empty_title_then_creation_fails() {
        // act
        let result = Issue::new(
            "  ".into(),
            FingerprintHash::new("abc123").unwrap(),
            ProjectId::new(),
            EventLevel::Error,
            Utc::now(),
        );

        // assert
        match result {
            Err(IssueError::EmptyTitle) => (),
            other => panic!("Expected EmptyTitle error, got {:?}", other),
        }
    }

    #[test]
    fn given_empty_fingerprint_hash_then_creation_fails() {
        // act
        let result = FingerprintHash::new("  ");

        // assert
        match result {
            Err(IssueError::EmptyFingerprintHash) => (),
            other => panic!("Expected EmptyFingerprintHash error, got {:?}", other),
        }
    }

    #[test]
    fn given_title_with_whitespace_then_title_is_trimmed() {
        // act
        let issue = Issue::new(
            "  Something broke  ".into(),
            FingerprintHash::new("abc123").unwrap(),
            ProjectId::new(),
            EventLevel::Error,
            Utc::now(),
        )
        .unwrap();

        // assert
        assert_eq!(issue.title(), "Something broke");
    }

    #[test]
    fn given_unresolved_issue_then_resolve_succeeds() {
        // arrange
        let mut issue = test_issue();

        // act
        issue.resolve().expect("Failed to resolve");

        // assert
        assert_eq!(issue.status(), &IssueStatus::Resolved);
    }

    #[test]
    fn given_resolved_issue_then_resolve_fails() {
        // arrange
        let mut issue = test_issue();
        issue.resolve().unwrap();

        // act
        let result = issue.resolve();

        // assert
        match result {
            Err(IssueError::AlreadyResolved) => (),
            other => panic!("Expected AlreadyResolved error, got {:?}", other),
        }
    }

    #[test]
    fn given_resolved_issue_then_reopen_succeeds() {
        // arrange
        let mut issue = test_issue();
        issue.resolve().unwrap();

        // act
        issue.reopen().expect("Failed to reopen");

        // assert
        assert_eq!(issue.status(), &IssueStatus::Unresolved);
    }

    #[test]
    fn given_unresolved_issue_then_reopen_fails() {
        // arrange
        let mut issue = test_issue();

        // act
        let result = issue.reopen();

        // assert
        match result {
            Err(IssueError::AlreadyUnresolved) => (),
            other => panic!("Expected AlreadyUnresolved error, got {:?}", other),
        }
    }

    #[test]
    fn given_unresolved_issue_then_ignore_succeeds() {
        // arrange
        let mut issue = test_issue();

        // act
        issue.ignore().expect("Failed to ignore");

        // assert
        assert_eq!(issue.status(), &IssueStatus::Ignored);
    }

    #[test]
    fn given_ignored_issue_then_ignore_fails() {
        // arrange
        let mut issue = test_issue();
        issue.ignore().unwrap();

        // act
        let result = issue.ignore();

        // assert
        match result {
            Err(IssueError::AlreadyIgnored) => (),
            other => panic!("Expected AlreadyIgnored error, got {:?}", other),
        }
    }

    #[test]
    fn given_event_recorded_then_count_increments() {
        // arrange
        let mut issue = test_issue();
        let later = Utc::now();

        // act
        issue.record_event(EventLevel::Error, later);

        // assert
        assert_eq!(issue.event_count(), 2);
    }

    #[test]
    fn given_event_recorded_then_last_seen_updates() {
        // arrange
        let mut issue = test_issue();
        let original_last_seen = *issue.last_seen();
        let later = original_last_seen + chrono::Duration::seconds(60);

        // act
        issue.record_event(EventLevel::Error, later);

        // assert
        assert_eq!(issue.last_seen(), &later);
        assert_eq!(issue.first_seen(), &original_last_seen);
    }

    #[test]
    fn given_older_event_then_last_seen_does_not_regress() {
        // arrange
        let mut issue = test_issue();
        let original_last_seen = *issue.last_seen();
        let earlier = original_last_seen - chrono::Duration::seconds(60);

        // act
        issue.record_event(EventLevel::Error, earlier);

        // assert
        assert_eq!(issue.last_seen(), &original_last_seen);
        assert_eq!(issue.event_count(), 2);
    }

    #[test]
    fn given_higher_severity_event_then_level_escalates() {
        // arrange
        let mut issue = Issue::new(
            "Something broke".into(),
            FingerprintHash::new("abc123").unwrap(),
            ProjectId::new(),
            EventLevel::Warning,
            Utc::now(),
        )
        .unwrap();

        // act
        issue.record_event(EventLevel::Fatal, Utc::now());

        // assert
        assert_eq!(issue.level(), &EventLevel::Fatal);
    }

    #[test]
    fn given_lower_severity_event_then_level_does_not_downgrade() {
        // arrange
        let mut issue = Issue::new(
            "Something broke".into(),
            FingerprintHash::new("abc123").unwrap(),
            ProjectId::new(),
            EventLevel::Fatal,
            Utc::now(),
        )
        .unwrap();

        // act
        issue.record_event(EventLevel::Info, Utc::now());

        // assert
        assert_eq!(issue.level(), &EventLevel::Fatal);
    }

    #[test]
    fn given_ignored_issue_then_reopen_succeeds() {
        // arrange
        let mut issue = test_issue();
        issue.ignore().unwrap();

        // act
        issue.reopen().expect("Failed to reopen ignored issue");

        // assert
        assert_eq!(issue.status(), &IssueStatus::Unresolved);
    }

    #[test]
    fn given_type_and_value_then_derive_title_formats_both() {
        // act
        let title = Issue::derive_title(Some("TypeError"), Some("x is not defined"), "fallback");

        // assert
        assert_eq!(title, "TypeError: x is not defined");
    }

    #[test]
    fn given_type_only_then_derive_title_uses_type() {
        // act
        let title = Issue::derive_title(Some("TypeError"), None, "fallback");

        // assert
        assert_eq!(title, "TypeError");
    }

    #[test]
    fn given_issue_number_then_display_formats_with_hash() {
        // act
        let number = IssueNumber::new(42);

        // assert
        assert_eq!(number.to_string(), "#42");
        assert_eq!(number.value(), 42);
    }

    #[test]
    fn given_no_exception_info_then_derive_title_uses_message() {
        // act
        let title = Issue::derive_title(None, None, "Something broke");

        // assert
        assert_eq!(title, "Something broke");
    }
}
