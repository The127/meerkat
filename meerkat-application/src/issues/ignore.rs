use async_trait::async_trait;

use meerkat_domain::models::issue::{IssueId, IssueIdentifier};
use meerkat_domain::models::permission::ProjectPermission;
use meerkat_domain::models::project::ProjectIdentifier;

use crate::behaviors::authorization::project_extensions;
use crate::context::RequestContext;
use crate::error::ApplicationError;
use crate::extensions::Extensions;
use crate::mediator::{Request, Handler};

pub struct IgnoreIssue {
    pub project: ProjectIdentifier,
    pub issue_id: IssueId,
}

impl Request for IgnoreIssue {
    type Output = ();

    fn extensions(&self) -> Extensions {
        project_extensions(
            "IgnoreIssue",
            vec![ProjectPermission::ProjectWrite.into()],
            self.project.clone(),
        )
    }
}

pub struct IgnoreIssueHandler;

#[async_trait]
impl Handler<IgnoreIssue, ApplicationError, RequestContext> for IgnoreIssueHandler {
    async fn handle(
        &self,
        cmd: IgnoreIssue,
        ctx: &RequestContext,
    ) -> Result<(), ApplicationError> {
        let uow = ctx.uow().await;
        let mut issue = uow
            .issues()
            .find(&IssueIdentifier::Id(cmd.issue_id))
            .await?;
        issue.ignore()?;
        uow.issues().save(issue);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use meerkat_domain::models::issue::IssueStatus;
    use meerkat_domain::models::organization::OrganizationId;
    use meerkat_domain::models::project::{ProjectIdentifier, ProjectSlug};
    use meerkat_domain::testing::test_issue;

    use crate::context::RequestContext;
    use crate::mediator::Handler;
    use crate::ports::issue_repository::MockIssueRepository;
    use crate::ports::unit_of_work::MockUnitOfWork;

    use super::{IgnoreIssue, IgnoreIssueHandler};

    #[tokio::test]
    async fn given_unresolved_issue_then_finds_ignores_and_saves() {
        // arrange
        let issue = test_issue();
        let issue_id = issue.id().clone();

        let mut issue_repo = MockIssueRepository::new();
        issue_repo.expect_find()
            .times(1)
            .returning(move |_| Box::pin(std::future::ready(Ok(issue.clone()))));
        issue_repo.expect_save()
            .times(1)
            .withf(|issue| *issue.status() == IssueStatus::Ignored)
            .returning(|_| ());

        let uow = MockUnitOfWork::new().with_issue_repo(issue_repo);
        let ctx = RequestContext::test()
            .with_scoped_uow(Box::new(uow));

        let handler = IgnoreIssueHandler;
        let cmd = IgnoreIssue {
            project: ProjectIdentifier::Slug(OrganizationId::new(), ProjectSlug::new("test-project").unwrap()),
            issue_id,
        };

        // act
        let result = handler.handle(cmd, &ctx).await;

        // assert
        assert!(result.is_ok());
    }
}
