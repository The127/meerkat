use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};

use meerkat_domain::models::event::{Event, EventId, EventLevel};
use meerkat_domain::models::issue::{Issue, IssueIdentifier};
use meerkat_domain::models::project::ProjectId;
use crate::behaviors::rate_limit::RateLimitKey;
use crate::context::RequestContext;
use crate::error::ApplicationError;
use crate::events::DomainEvent;
use mediator_rs::{Extensions, Request, Handler};
use crate::ports::fingerprint_service::FingerprintService;

pub struct IngestEvent {
    pub project_id: ProjectId,
    pub rate_limit_key: RateLimitKey,
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

    fn extensions(&self) -> Extensions {
        let mut ext = Extensions::new();
        ext.insert(RateLimitKey {
            key_token: self.rate_limit_key.key_token.clone(),
            max_per_window: self.rate_limit_key.max_per_window,
        });
        ext
    }
}

pub struct IngestEventHandler {
    fingerprint_service: Arc<dyn FingerprintService>,
}

impl IngestEventHandler {
    pub fn new(fingerprint_service: Arc<dyn FingerprintService>) -> Self {
        Self { fingerprint_service }
    }
}

#[async_trait]
impl Handler<IngestEvent, ApplicationError, RequestContext> for IngestEventHandler {
    async fn handle(&self, cmd: IngestEvent, ctx: &RequestContext) -> Result<EventId, ApplicationError> {
        let fingerprint_hash = self.fingerprint_service.compute(
            cmd.exception_type.clone(),
            cmd.exception_value.clone(),
            cmd.message.clone(),
        );

        let title = Issue::derive_title(
            cmd.exception_type.as_deref(),
            cmd.exception_value.as_deref(),
            &cmd.message,
        );

        let uow = ctx.uow().await;
        let existing_issue = match uow
            .issues()
            .find(&IssueIdentifier::Fingerprint(cmd.project_id.clone(), fingerprint_hash.clone()))
            .await
        {
            Ok(issue) => Some(issue),
            Err(ApplicationError::NotFound) => None,
            Err(e) => return Err(e),
        };

        let issue_id = match existing_issue {
            Some(mut issue) => {
                issue.record_event(cmd.level.clone(), cmd.timestamp);
                let id = issue.id().clone();
                uow.issues().save(issue);
                ctx.raise(DomainEvent::EventRecorded { issue_id: id.clone() }).await;
                id
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
                uow.issues().add(issue);
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
        uow.events().add(event);

        Ok(event_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::RequestContext;
    use mediator_rs::Handler;
    use crate::ports::event_repository::MockEventRepository;
    use crate::ports::fingerprint_service::MockFingerprintService;
    use crate::ports::issue_repository::MockIssueRepository;
    use crate::ports::unit_of_work::MockUnitOfWork;
    use meerkat_domain::models::event::EventLevel;
    use meerkat_domain::models::issue::FingerprintHash;
    use meerkat_domain::models::project::ProjectId;

    fn test_command() -> IngestEvent {
        IngestEvent {
            project_id: ProjectId::new(),
            rate_limit_key: RateLimitKey {
                key_token: "test-key-token".to_string(),
                max_per_window: None,
            },
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

    fn test_fingerprint_service() -> MockFingerprintService {
        let mut svc = MockFingerprintService::new();
        svc.expect_compute()
            .returning(|_, _, _| FingerprintHash::new("test-fingerprint").unwrap());
        svc
    }

    fn test_event_repo() -> MockEventRepository {
        let mut repo = MockEventRepository::new();
        repo.expect_add().returning(|_| ());
        repo
    }

    fn test_existing_issue(project_id: &ProjectId) -> Issue {
        Issue::new(
            "TypeError: x is not defined".into(),
            FingerprintHash::new("test-fingerprint").unwrap(),
            project_id.clone(),
            EventLevel::Warning,
            Utc::now() - chrono::Duration::hours(1),
        )
        .unwrap()
    }

    fn build_handler_and_ctx(
        issue_repo: MockIssueRepository,
        event_repo: MockEventRepository,
        fingerprint_svc: MockFingerprintService,
    ) -> (IngestEventHandler, RequestContext) {
        let uow = MockUnitOfWork::new()
            .with_event_repo(event_repo)
            .with_issue_repo(issue_repo);
        let handler = IngestEventHandler::new(Arc::new(fingerprint_svc));
        let ctx = RequestContext::test()
            .with_scoped_uow(Box::new(uow));
        (handler, ctx)
    }

    #[tokio::test]
    async fn given_new_fingerprint_then_creates_new_issue() {
        // arrange
        let mut issue_repo = MockIssueRepository::new();
        issue_repo.expect_find()
            .times(1)
            .returning(|_| Box::pin(std::future::ready(Err(ApplicationError::NotFound))));
        issue_repo.expect_add()
            .times(1)
            .withf(|issue: &Issue| {
                issue.title() == "TypeError: x is not defined"
                    && issue.event_count() == 1
            })
            .returning(|_| ());

        let mut event_repo = MockEventRepository::new();
        event_repo.expect_add()
            .times(1)
            .withf(|event: &Event| {
                event.message() == "Something broke"
                    && event.platform() == "python"
            })
            .returning(|_| ());

        let (handler, ctx) = build_handler_and_ctx(
            issue_repo, event_repo, test_fingerprint_service(),
        );

        // act
        let result = handler.handle(test_command(), &ctx).await;

        // assert
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn given_existing_fingerprint_then_records_event_on_existing_issue() {
        // arrange
        let cmd = test_command();
        let existing = test_existing_issue(&cmd.project_id);

        let mut issue_repo = MockIssueRepository::new();
        issue_repo.expect_find()
            .returning(move |_| Box::pin(std::future::ready(Ok(existing.clone()))));
        issue_repo.expect_save()
            .times(1)
            .withf(|issue: &Issue| issue.event_count() == 2)
            .returning(|_| ());

        let (handler, ctx) = build_handler_and_ctx(
            issue_repo, test_event_repo(), test_fingerprint_service(),
        );

        // act
        let result = handler.handle(test_command(), &ctx).await;

        // assert
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn given_existing_fingerprint_then_raises_event_recorded() {
        // arrange
        let cmd = test_command();
        let existing = test_existing_issue(&cmd.project_id);
        let expected_issue_id = existing.id().clone();

        let mut issue_repo = MockIssueRepository::new();
        issue_repo.expect_find()
            .returning(move |_| Box::pin(std::future::ready(Ok(existing.clone()))));
        issue_repo.expect_save()
            .returning(|_| ());

        let (handler, ctx) = build_handler_and_ctx(
            issue_repo, test_event_repo(), test_fingerprint_service(),
        );

        // act
        handler.handle(test_command(), &ctx).await.unwrap();

        // assert
        let events = ctx.drain_events().await;
        assert_eq!(events.len(), 1);
        match &events[0] {
            DomainEvent::EventRecorded { issue_id } => {
                assert_eq!(issue_id, &expected_issue_id);
            }
            other => panic!("Expected EventRecorded, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn given_new_fingerprint_then_does_not_raise_event_recorded() {
        // arrange
        let mut issue_repo = MockIssueRepository::new();
        issue_repo.expect_find()
            .returning(|_| Box::pin(std::future::ready(Err(ApplicationError::NotFound))));
        issue_repo.expect_add()
            .returning(|_| ());

        let (handler, ctx) = build_handler_and_ctx(
            issue_repo, test_event_repo(), test_fingerprint_service(),
        );

        // act
        handler.handle(test_command(), &ctx).await.unwrap();

        // assert
        let events = ctx.drain_events().await;
        assert!(events.is_empty());
    }
}
