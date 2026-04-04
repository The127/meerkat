use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};

use meerkat_domain::models::event::{Event, EventId, EventLevel};
use meerkat_domain::models::issue::Issue;
use meerkat_domain::models::project::ProjectId;

use crate::context::RequestContext;
use crate::error::ApplicationError;
use crate::mediator::{Request, Handler};
use crate::ports::event_repository::EventRepository;
use crate::ports::fingerprint_service::FingerprintService;
use crate::ports::issue_repository::IssueRepository;

pub struct IngestEvent {
    pub project_id: ProjectId,
    pub message: String,
    pub level: EventLevel,
    pub platform: String,
    pub timestamp: DateTime<Utc>,
    pub server_name: Option<String>,
    pub environment: Option<String>,
    pub release: Option<String>,
    pub exception_type: Option<String>,
    pub exception_value: Option<String>,
    pub tags: Vec<(String, String)>,
    pub extra: serde_json::Value,
}

impl Request for IngestEvent {
    type Output = EventId;
}

pub struct IngestEventHandler {
    event_repo: Arc<dyn EventRepository>,
    issue_repo: Arc<dyn IssueRepository>,
    fingerprint_service: Arc<dyn FingerprintService>,
}

impl IngestEventHandler {
    pub fn new(
        event_repo: Arc<dyn EventRepository>,
        issue_repo: Arc<dyn IssueRepository>,
        fingerprint_service: Arc<dyn FingerprintService>,
    ) -> Self {
        Self {
            event_repo,
            issue_repo,
            fingerprint_service,
        }
    }
}

#[async_trait]
impl Handler<IngestEvent, ApplicationError, RequestContext> for IngestEventHandler {
    async fn handle(&self, cmd: IngestEvent, _ctx: &RequestContext) -> Result<EventId, ApplicationError> {
        let fingerprint_hash = self.fingerprint_service.compute(
            cmd.exception_type.clone(),
            cmd.exception_value.clone(),
            cmd.message.clone(),
        );

        let title = match (&cmd.exception_type, &cmd.exception_value) {
            (Some(t), Some(v)) => format!("{t}: {v}"),
            (Some(t), None) => t.clone(),
            _ => cmd.message.clone(),
        };

        let existing_issue = self
            .issue_repo
            .find_by_fingerprint(&cmd.project_id, &fingerprint_hash)
            .await?;

        let issue_id = match existing_issue {
            Some(mut issue) => {
                issue.record_event(cmd.level.clone(), cmd.timestamp);
                self.issue_repo.save(&issue).await?;
                issue.id().clone()
            }
            None => {
                let issue = Issue::new(
                    title,
                    fingerprint_hash.clone(),
                    cmd.project_id.clone(),
                    cmd.level.clone(),
                    cmd.timestamp,
                )?;
                let id = issue.id().clone();
                self.issue_repo.add(&issue).await?;
                id
            }
        };

        let event = Event::new(
            cmd.project_id,
            issue_id,
            fingerprint_hash,
            cmd.message,
            cmd.level,
            cmd.platform,
            cmd.timestamp,
            cmd.server_name,
            cmd.environment,
            cmd.release,
            cmd.exception_type,
            cmd.exception_value,
            cmd.tags,
            cmd.extra,
        )?;

        let event_id = event.id().clone();
        self.event_repo.add(&event).await?;

        Ok(event_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::RequestContext;
    use crate::mediator::Handler;
    use crate::ports::event_repository::MockEventRepository;
    use crate::ports::fingerprint_service::MockFingerprintService;
    use crate::ports::issue_repository::MockIssueRepository;
    use meerkat_domain::models::event::EventLevel;
    use meerkat_domain::models::project::ProjectId;

    fn test_command() -> IngestEvent {
        IngestEvent {
            project_id: ProjectId::new(),
            message: "Something broke".into(),
            level: EventLevel::Error,
            platform: "python".into(),
            timestamp: Utc::now(),
            server_name: None,
            environment: Some("production".into()),
            release: None,
            exception_type: Some("TypeError".into()),
            exception_value: Some("x is not defined".into()),
            tags: vec![],
            extra: serde_json::Value::Null,
        }
    }

    #[tokio::test]
    async fn given_new_fingerprint_then_creates_new_issue() {
        // arrange
        let mut issue_repo = MockIssueRepository::new();
        issue_repo.expect_find_by_fingerprint()
            .times(1)
            .returning(|_, _| Box::pin(std::future::ready(Ok(None))));
        issue_repo.expect_add()
            .times(1)
            .withf(|issue| {
                issue.title() == "TypeError: x is not defined"
                    && issue.event_count() == 1
            })
            .returning(|_| Box::pin(std::future::ready(Ok(()))));

        let mut event_repo = MockEventRepository::new();
        event_repo.expect_add()
            .times(1)
            .withf(|event| {
                event.message() == "Something broke"
                    && event.platform() == "python"
            })
            .returning(|_| Box::pin(std::future::ready(Ok(()))));

        let mut fingerprint_svc = MockFingerprintService::new();
        fingerprint_svc.expect_compute()
            .returning(|_, _, _| "test-fingerprint".to_string());

        let handler = IngestEventHandler::new(Arc::new(event_repo), Arc::new(issue_repo), Arc::new(fingerprint_svc));
        let ctx = RequestContext::test();

        // act
        let result = handler.handle(test_command(), &ctx).await;

        // assert
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn given_existing_fingerprint_then_records_event_on_existing_issue() {
        // arrange
        let cmd = test_command();
        let existing = Issue::new(
            "TypeError: x is not defined".into(),
            "test-fingerprint".into(),
            cmd.project_id.clone(),
            EventLevel::Warning,
            Utc::now() - chrono::Duration::hours(1),
        )
        .unwrap();

        let mut issue_repo = MockIssueRepository::new();
        issue_repo.expect_find_by_fingerprint()
            .times(1)
            .returning(move |_, _| Box::pin(std::future::ready(Ok(Some(existing.clone())))));
        issue_repo.expect_save()
            .times(1)
            .withf(|issue| issue.event_count() == 2)
            .returning(|_| Box::pin(std::future::ready(Ok(()))));

        let mut event_repo = MockEventRepository::new();
        event_repo.expect_add()
            .times(1)
            .returning(|_| Box::pin(std::future::ready(Ok(()))));

        let mut fingerprint_svc = MockFingerprintService::new();
        fingerprint_svc.expect_compute()
            .returning(|_, _, _| "test-fingerprint".to_string());

        let handler = IngestEventHandler::new(Arc::new(event_repo), Arc::new(issue_repo), Arc::new(fingerprint_svc));
        let ctx = RequestContext::test();

        // act
        let result = handler.handle(test_command(), &ctx).await;

        // assert
        assert!(result.is_ok());
    }
}
