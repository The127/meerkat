use chrono::{DateTime, Utc};
use meerkat_macros::{uuid_id, Reconstitute};
use crate::models::project::ProjectId;

uuid_id!(EventId);

#[derive(Debug, Clone, Default, PartialEq, Eq, strum::Display, strum::EnumString, strum::AsRefStr)]
pub enum EventLevel {
    #[strum(serialize = "fatal")]
    Fatal,
    #[strum(serialize = "error")]
    #[default]
    Error,
    #[strum(serialize = "warning")]
    Warning,
    #[strum(serialize = "info")]
    Info,
    #[strum(serialize = "debug")]
    Debug,
}

impl EventLevel {
    pub fn severity(&self) -> u8 {
        match self {
            EventLevel::Debug => 0,
            EventLevel::Info => 1,
            EventLevel::Warning => 2,
            EventLevel::Error => 3,
            EventLevel::Fatal => 4,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum EventError {
    #[error("event message must not be empty")]
    EmptyMessage,
    #[error("event platform must not be empty")]
    EmptyPlatform,
}

#[derive(Debug, Clone, Reconstitute)]
pub struct Event {
    id: EventId,
    project_id: ProjectId,
    fingerprint_hash: String,
    message: String,
    level: EventLevel,
    platform: String,
    timestamp: DateTime<Utc>,
    server_name: Option<String>,
    environment: Option<String>,
    release: Option<String>,
    exception_type: Option<String>,
    exception_value: Option<String>,
    tags: Vec<(String, String)>,
    extra: serde_json::Value,
}

impl Event {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        project_id: ProjectId,
        fingerprint_hash: String,
        message: String,
        level: EventLevel,
        platform: String,
        timestamp: DateTime<Utc>,
        server_name: Option<String>,
        environment: Option<String>,
        release: Option<String>,
        exception_type: Option<String>,
        exception_value: Option<String>,
        tags: Vec<(String, String)>,
        extra: serde_json::Value,
    ) -> Result<Self, EventError> {
        let message = message.trim().to_string();
        if message.is_empty() {
            return Err(EventError::EmptyMessage);
        }

        let platform = platform.trim().to_string();
        if platform.is_empty() {
            return Err(EventError::EmptyPlatform);
        }

        let fingerprint_hash = fingerprint_hash.trim().to_string();

        Ok(Self {
            id: EventId::new(),
            project_id,
            fingerprint_hash,
            message,
            level,
            platform,
            timestamp,
            server_name,
            environment,
            release,
            exception_type,
            exception_value,
            tags,
            extra,
        })
    }

    pub fn id(&self) -> &EventId { &self.id }
    pub fn project_id(&self) -> &ProjectId { &self.project_id }
    pub fn fingerprint_hash(&self) -> &str { &self.fingerprint_hash }
    pub fn message(&self) -> &str { &self.message }
    pub fn level(&self) -> &EventLevel { &self.level }
    pub fn platform(&self) -> &str { &self.platform }
    pub fn timestamp(&self) -> &DateTime<Utc> { &self.timestamp }
    pub fn server_name(&self) -> Option<&str> { self.server_name.as_deref() }
    pub fn environment(&self) -> Option<&str> { self.environment.as_deref() }
    pub fn release(&self) -> Option<&str> { self.release.as_deref() }
    pub fn exception_type(&self) -> Option<&str> { self.exception_type.as_deref() }
    pub fn exception_value(&self) -> Option<&str> { self.exception_value.as_deref() }
    pub fn tags(&self) -> &[(String, String)] { &self.tags }
    pub fn extra(&self) -> &serde_json::Value { &self.extra }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::test_event;

    #[test]
    fn given_valid_input_then_creation_succeeds() {
        // arrange
        let project_id = ProjectId::new();

        // act
        let event = Event::new(
            project_id.clone(),
            "abc123".into(),
            "Something broke".into(),
            EventLevel::Error,
            "javascript".into(),
            Utc::now(),
            None,
            Some("production".into()),
            None,
            Some("TypeError".into()),
            Some("Cannot read property 'x'".into()),
            vec![("browser".into(), "chrome".into())],
            serde_json::json!({"user_id": 42}),
        )
        .expect("Failed to create event");

        // assert
        assert_eq!(event.project_id(), &project_id);
        assert_eq!(event.message(), "Something broke");
        assert_eq!(event.level(), &EventLevel::Error);
        assert_eq!(event.platform(), "javascript");
        assert_eq!(event.environment(), Some("production"));
        assert_eq!(event.exception_type(), Some("TypeError"));
        assert_eq!(event.tags().len(), 1);
    }

    #[test]
    fn given_empty_message_then_creation_fails() {
        // act
        let result = Event::new(
            ProjectId::new(),
            "abc123".into(),
            "  ".into(),
            EventLevel::Error,
            "python".into(),
            Utc::now(),
            None,
            None,
            None,
            None,
            None,
            vec![],
            serde_json::Value::Null,
        );

        // assert
        match result {
            Err(EventError::EmptyMessage) => (),
            other => panic!("Expected EmptyMessage error, got {:?}", other),
        }
    }

    #[test]
    fn given_empty_platform_then_creation_fails() {
        // act
        let result = Event::new(
            ProjectId::new(),
            "abc123".into(),
            "Something broke".into(),
            EventLevel::Error,
            "  ".into(),
            Utc::now(),
            None,
            None,
            None,
            None,
            None,
            vec![],
            serde_json::Value::Null,
        );

        // assert
        match result {
            Err(EventError::EmptyPlatform) => (),
            other => panic!("Expected EmptyPlatform error, got {:?}", other),
        }
    }

    #[test]
    fn given_message_with_whitespace_then_message_is_trimmed() {
        // act
        let event = Event::new(
            ProjectId::new(),
            "abc123".into(),
            "  Something broke  ".into(),
            EventLevel::Error,
            "python".into(),
            Utc::now(),
            None,
            None,
            None,
            None,
            None,
            vec![],
            serde_json::Value::Null,
        )
        .unwrap();

        // assert
        assert_eq!(event.message(), "Something broke");
    }

    #[test]
    fn given_default_level_then_level_is_error() {
        // act
        let level = EventLevel::default();

        // assert
        assert_eq!(level, EventLevel::Error);
    }

    #[test]
    fn given_test_helper_then_event_is_valid() {
        // act
        let event = test_event();

        // assert
        assert_eq!(event.message(), "Test error");
        assert_eq!(event.platform(), "python");
    }
}
