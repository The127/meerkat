use async_trait::async_trait;

use meerkat_domain::models::issue::{IssueId, IssueIdentifier};
use meerkat_domain::models::permission::ProjectPermission;
use meerkat_domain::models::project::ProjectIdentifier;

use crate::behaviors::authorization::project_extensions;
use crate::context::RequestContext;
use crate::error::ApplicationError;
use crate::extensions::Extensions;
use crate::mediator::{Request, Handler};

pub struct ResolveIssue {
    pub project: ProjectIdentifier,
    pub issue_id: IssueId,
}

impl Request for ResolveIssue {
    type Output = ();

    fn extensions(&self) -> Extensions {
        project_extensions(
            "ResolveIssue",
            vec![ProjectPermission::ProjectWrite.into()],
            self.project.clone(),
        )
    }
}

pub struct ResolveIssueHandler;

#[async_trait]
impl Handler<ResolveIssue, ApplicationError, RequestContext> for ResolveIssueHandler {
    async fn handle(
        &self,
        cmd: ResolveIssue,
        ctx: &RequestContext,
    ) -> Result<(), ApplicationError> {
        let uow = ctx.uow().await;
        let mut issue = uow
            .issues()
            .find(&IssueIdentifier::Id(cmd.issue_id))
            .await?;
        issue.resolve()?;
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

    use super::{ResolveIssue, ResolveIssueHandler};

    #[tokio::test]
    async fn given_unresolved_issue_then_finds_resolves_and_saves() {
        // arrange
        let issue = test_issue();
        let issue_id = issue.id().clone();

        let mut issue_repo = MockIssueRepository::new();
        issue_repo.expect_find()
            .times(1)
            .returning(move |_| Box::pin(std::future::ready(Ok(issue.clone()))));
        issue_repo.expect_save()
            .times(1)
            .withf(|issue| *issue.status() == IssueStatus::Resolved)
            .returning(|_| ());

        let uow = MockUnitOfWork::new().with_issue_repo(issue_repo);
        let ctx = RequestContext::test()
            .with_scoped_uow(Box::new(uow));

        let handler = ResolveIssueHandler;
        let cmd = ResolveIssue {
            project: ProjectIdentifier::Slug(OrganizationId::new(), ProjectSlug::new("test-project").unwrap()),
            issue_id,
        };

        // act
        let result = handler.handle(cmd, &ctx).await;

        // assert
        assert!(result.is_ok());
    }
}
