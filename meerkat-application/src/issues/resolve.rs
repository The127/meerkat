use async_trait::async_trait;

use meerkat_domain::models::issue::{IssueIdentifier, IssueNumber};
use meerkat_domain::models::permission::ProjectPermission;
use meerkat_domain::models::project::ProjectIdentifier;

use crate::behaviors::authorization::project_extensions;
use crate::context::RequestContext;
use crate::error::ApplicationError;
use mediator_rs::{Extensions, Request, Handler};
use crate::ports::project_read_store::ProjectReadStore;

pub struct ResolveIssue {
    pub project: ProjectIdentifier,
    pub issue_number: IssueNumber,
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

pub struct ResolveIssueHandler {
    project_read_store: std::sync::Arc<dyn ProjectReadStore>,
}

impl ResolveIssueHandler {
    pub fn new(project_read_store: std::sync::Arc<dyn ProjectReadStore>) -> Self {
        Self { project_read_store }
    }
}

#[async_trait]
impl Handler<ResolveIssue, ApplicationError, RequestContext> for ResolveIssueHandler {
    async fn handle(
        &self,
        cmd: ResolveIssue,
        ctx: &RequestContext,
    ) -> Result<(), ApplicationError> {
        let ProjectIdentifier::Slug(ref org_id, ref slug) = cmd.project else {
            return Err(ApplicationError::NotFound);
        };
        let project = self.project_read_store
            .find_by_slug(org_id, slug)
            .await?
            .ok_or(ApplicationError::NotFound)?;

        let uow = ctx.uow().await;
        let mut issue = uow
            .issues()
            .find(&IssueIdentifier::Number(project.id, cmd.issue_number))
            .await?;
        issue.resolve()?;
        uow.issues().save(issue);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use meerkat_domain::models::issue::{IssueNumber, IssueStatus};
    use meerkat_domain::models::organization::OrganizationId;
    use meerkat_domain::models::project::{ProjectIdentifier, ProjectSlug};
    use meerkat_domain::testing::test_issue;

    use crate::context::RequestContext;
    use mediator_rs::Handler;
    use crate::ports::issue_repository::MockIssueRepository;
    use crate::ports::project_read_store::MockProjectReadStore;
    use crate::ports::unit_of_work::MockUnitOfWork;

    use super::{ResolveIssue, ResolveIssueHandler};

    #[tokio::test]
    async fn given_unresolved_issue_then_finds_resolves_and_saves() {
        // arrange
        let issue = test_issue();
        let org_id = OrganizationId::new();

        let mut project_store = MockProjectReadStore::new();
        let org_id_clone = org_id.clone();
        project_store.expect_find_by_slug()
            .times(1)
            .withf(move |o, s| *o == org_id_clone && s.as_str() == "test-project")
            .returning(|_, _| {
                Box::pin(std::future::ready(Ok(Some(
                    crate::ports::project_read_store::ProjectReadModel {
                        id: meerkat_domain::models::project::ProjectId::new(),
                        organization_id: OrganizationId::new(),
                        name: "Test Project".to_string(),
                        slug: ProjectSlug::new("test-project").unwrap(),
                        created_at: chrono::Utc::now(),
                        updated_at: chrono::Utc::now(),
                    }
                ))))
            });

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

        let handler = ResolveIssueHandler::new(std::sync::Arc::new(project_store));
        let cmd = ResolveIssue {
            project: ProjectIdentifier::Slug(org_id, ProjectSlug::new("test-project").unwrap()),
            issue_number: IssueNumber::new(1),
        };

        // act
        let result = handler.handle(cmd, &ctx).await;

        // assert
        assert!(result.is_ok());
    }
}
