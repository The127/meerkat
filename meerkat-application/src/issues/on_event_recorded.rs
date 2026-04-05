use async_trait::async_trait;

use meerkat_domain::models::issue::{IssueIdentifier, IssueStatus};

use crate::context::RequestContext;
use crate::error::ApplicationError;
use crate::events::{DomainEvent, DomainEventHandler};

pub struct RegressResolvedIssueOnNewEvent;

#[async_trait]
impl DomainEventHandler for RegressResolvedIssueOnNewEvent {
    async fn handle(&self, event: &DomainEvent, ctx: &RequestContext) -> Result<(), ApplicationError> {
        let DomainEvent::EventRecorded { issue_id } = event else {
            return Ok(());
        };

        let uow = ctx.uow().await;
        let mut issue = uow
            .issues()
            .find(&IssueIdentifier::Id(issue_id.clone()))
            .await?;

        if *issue.status() != IssueStatus::Resolved {
            return Ok(());
        }

        issue.regress()?;
        uow.issues().save(issue);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use meerkat_domain::models::issue::IssueStatus;
    use meerkat_domain::testing::test_issue;

    use crate::context::RequestContext;
    use crate::events::{DomainEvent, DomainEventHandler};
    use crate::ports::issue_repository::MockIssueRepository;
    use crate::ports::unit_of_work::MockUnitOfWork;

    use super::RegressResolvedIssueOnNewEvent;

    #[tokio::test]
    async fn given_resolved_issue_then_regresses() {
        // arrange
        let mut issue = test_issue();
        issue.resolve().unwrap();
        let issue_id = issue.id().clone();

        let mut issue_repo = MockIssueRepository::new();
        issue_repo.expect_find()
            .times(1)
            .returning(move |_| Box::pin(std::future::ready(Ok(issue.clone()))));
        issue_repo.expect_save()
            .times(1)
            .withf(|issue| *issue.status() == IssueStatus::Regressed)
            .returning(|_| ());

        let uow = MockUnitOfWork::new().with_issue_repo(issue_repo);
        let ctx = RequestContext::test()
            .with_scoped_uow(Box::new(uow));

        let handler = RegressResolvedIssueOnNewEvent;
        let event = DomainEvent::EventRecorded { issue_id };

        // act
        let result = handler.handle(&event, &ctx).await;

        // assert
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn given_unresolved_issue_then_does_nothing() {
        // arrange
        let issue = test_issue();
        let issue_id = issue.id().clone();

        let mut issue_repo = MockIssueRepository::new();
        issue_repo.expect_find()
            .times(1)
            .returning(move |_| Box::pin(std::future::ready(Ok(issue.clone()))));
        issue_repo.expect_save()
            .never()
            .returning(|_| ());

        let uow = MockUnitOfWork::new().with_issue_repo(issue_repo);
        let ctx = RequestContext::test()
            .with_scoped_uow(Box::new(uow));

        let handler = RegressResolvedIssueOnNewEvent;
        let event = DomainEvent::EventRecorded { issue_id };

        // act
        let result = handler.handle(&event, &ctx).await;

        // assert
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn given_ignored_issue_then_does_nothing() {
        // arrange
        let mut issue = test_issue();
        issue.ignore().unwrap();
        let issue_id = issue.id().clone();

        let mut issue_repo = MockIssueRepository::new();
        issue_repo.expect_find()
            .times(1)
            .returning(move |_| Box::pin(std::future::ready(Ok(issue.clone()))));
        issue_repo.expect_save()
            .never()
            .returning(|_| ());

        let uow = MockUnitOfWork::new().with_issue_repo(issue_repo);
        let ctx = RequestContext::test()
            .with_scoped_uow(Box::new(uow));

        let handler = RegressResolvedIssueOnNewEvent;
        let event = DomainEvent::EventRecorded { issue_id };

        // act
        let result = handler.handle(&event, &ctx).await;

        // assert
        assert!(result.is_ok());
    }
}
