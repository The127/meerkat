use std::sync::Arc;

use async_trait::async_trait;

use meerkat_domain::models::issue::IssueId;
use meerkat_domain::models::permission::ProjectPermission;
use meerkat_domain::models::project::ProjectIdentifier;

use crate::behaviors::authorization::project_extensions;
use crate::context::RequestContext;
use crate::error::ApplicationError;
use crate::extensions::Extensions;
use crate::mediator::{Request, Handler};
use crate::ports::issue_read_store::{IssueReadModel, IssueReadStore};
use crate::ports::project_read_store::ProjectReadStore;

pub struct GetIssue {
    pub project: ProjectIdentifier,
    pub issue_id: IssueId,
}

impl Request for GetIssue {
    type Output = IssueReadModel;

    fn extensions(&self) -> Extensions {
        project_extensions(
            "GetIssue",
            vec![ProjectPermission::ProjectRead.into()],
            self.project.clone(),
        )
    }
}

pub struct GetIssueHandler {
    project_read_store: Arc<dyn ProjectReadStore>,
    issue_read_store: Arc<dyn IssueReadStore>,
}

impl GetIssueHandler {
    pub fn new(
        project_read_store: Arc<dyn ProjectReadStore>,
        issue_read_store: Arc<dyn IssueReadStore>,
    ) -> Self {
        Self { project_read_store, issue_read_store }
    }
}

#[async_trait]
impl Handler<GetIssue, ApplicationError, RequestContext> for GetIssueHandler {
    async fn handle(
        &self,
        cmd: GetIssue,
        _ctx: &RequestContext,
    ) -> Result<IssueReadModel, ApplicationError> {
        let ProjectIdentifier::Slug(ref org_id, ref slug) = cmd.project else {
            return Err(ApplicationError::NotFound);
        };
        // Resolve project to ensure it exists and authorization can check membership
        let project = self.project_read_store
            .find_by_slug(org_id, slug)
            .await?
            .ok_or(ApplicationError::NotFound)?;

        let issue = self.issue_read_store
            .find_by_id(&cmd.issue_id)
            .await?
            .ok_or(ApplicationError::NotFound)?;

        if issue.project_id != project.id {
            return Err(ApplicationError::NotFound);
        }

        Ok(issue)
    }
}

#[cfg(test)]
mod tests {
    use meerkat_domain::models::issue::IssueId;
    use meerkat_domain::models::organization::OrganizationId;
    use meerkat_domain::models::project::{ProjectIdentifier, ProjectSlug};

    use crate::context::RequestContext;
    use crate::mediator::Handler;
    use crate::ports::issue_read_store::{IssueReadModel, MockIssueReadStore};
    use crate::ports::project_read_store::MockProjectReadStore;

    use super::{GetIssue, GetIssueHandler};

    fn test_issue_read_model(issue_id: IssueId, project_id: meerkat_domain::models::project::ProjectId) -> IssueReadModel {
        IssueReadModel {
            id: issue_id,
            project_id,
            title: "TypeError: x is not defined".to_string(),
            fingerprint_hash: "abc123".to_string(),
            status: "unresolved".to_string(),
            level: "error".to_string(),
            event_count: 5,
            first_seen: chrono::Utc::now(),
            last_seen: chrono::Utc::now(),
        }
    }

    #[tokio::test]
    async fn given_existing_issue_then_returns_read_model() {
        // arrange
        let org_id = OrganizationId::new();
        let project_id = meerkat_domain::models::project::ProjectId::new();
        let slug = ProjectSlug::new("test-project").unwrap();
        let issue_id = IssueId::new();
        let expected = test_issue_read_model(issue_id.clone(), project_id.clone());

        let mut project_store = MockProjectReadStore::new();
        let org_id_clone = org_id.clone();
        let project_id_clone = project_id.clone();
        project_store.expect_find_by_slug()
            .times(1)
            .withf(move |o, s| *o == org_id_clone && s.as_str() == "test-project")
            .returning(move |_, _| {
                let pid = project_id_clone.clone();
                Box::pin(std::future::ready(Ok(Some(
                    crate::ports::project_read_store::ProjectReadModel {
                        id: pid,
                        organization_id: OrganizationId::new(),
                        name: "Test Project".to_string(),
                        slug: ProjectSlug::new("test-project").unwrap(),
                        created_at: chrono::Utc::now(),
                        updated_at: chrono::Utc::now(),
                    }
                ))))
            });

        let mut issue_store = MockIssueReadStore::new();
        let issue_id_clone = issue_id.clone();
        let expected_clone = expected.clone();
        issue_store.expect_find_by_id()
            .times(1)
            .withf(move |id| *id == issue_id_clone)
            .returning(move |_| Box::pin(std::future::ready(Ok(Some(expected_clone.clone())))));

        let handler = GetIssueHandler::new(
            std::sync::Arc::new(project_store),
            std::sync::Arc::new(issue_store),
        );
        let cmd = GetIssue {
            project: ProjectIdentifier::Slug(org_id, slug),
            issue_id,
        };
        let ctx = RequestContext::test();

        // act
        let result = handler.handle(cmd, &ctx).await;

        // assert
        let issue = result.unwrap();
        assert_eq!(issue.title, expected.title);
    }

    #[tokio::test]
    async fn given_nonexistent_issue_then_returns_not_found() {
        // arrange
        let org_id = OrganizationId::new();
        let slug = ProjectSlug::new("test-project").unwrap();

        let mut project_store = MockProjectReadStore::new();
        project_store.expect_find_by_slug()
            .times(1)
            .returning(|_, _| Box::pin(std::future::ready(Ok(Some(
                crate::ports::project_read_store::ProjectReadModel {
                    id: meerkat_domain::models::project::ProjectId::new(),
                    organization_id: OrganizationId::new(),
                    name: "Test Project".to_string(),
                    slug: ProjectSlug::new("test-project").unwrap(),
                    created_at: chrono::Utc::now(),
                    updated_at: chrono::Utc::now(),
                }
            )))));

        let mut issue_store = MockIssueReadStore::new();
        issue_store.expect_find_by_id()
            .times(1)
            .returning(|_| Box::pin(std::future::ready(Ok(None))));

        let handler = GetIssueHandler::new(
            std::sync::Arc::new(project_store),
            std::sync::Arc::new(issue_store),
        );
        let cmd = GetIssue {
            project: ProjectIdentifier::Slug(org_id, slug),
            issue_id: IssueId::new(),
        };
        let ctx = RequestContext::test();

        // act
        let result = handler.handle(cmd, &ctx).await;

        // assert
        assert!(matches!(result, Err(crate::error::ApplicationError::NotFound)));
    }

    #[tokio::test]
    async fn given_issue_from_different_project_then_returns_not_found() {
        // arrange
        let org_id = OrganizationId::new();
        let project_id = meerkat_domain::models::project::ProjectId::new();
        let other_project_id = meerkat_domain::models::project::ProjectId::new();
        let slug = ProjectSlug::new("test-project").unwrap();
        let issue_id = IssueId::new();
        let issue = test_issue_read_model(issue_id.clone(), other_project_id);

        let mut project_store = MockProjectReadStore::new();
        let org_id_clone = org_id.clone();
        project_store.expect_find_by_slug()
            .times(1)
            .withf(move |o, s| *o == org_id_clone && s.as_str() == "test-project")
            .returning(move |_, _| {
                let pid = project_id.clone();
                Box::pin(std::future::ready(Ok(Some(
                    crate::ports::project_read_store::ProjectReadModel {
                        id: pid,
                        organization_id: OrganizationId::new(),
                        name: "Test Project".to_string(),
                        slug: ProjectSlug::new("test-project").unwrap(),
                        created_at: chrono::Utc::now(),
                        updated_at: chrono::Utc::now(),
                    }
                ))))
            });

        let mut issue_store = MockIssueReadStore::new();
        let issue_clone = issue.clone();
        issue_store.expect_find_by_id()
            .times(1)
            .returning(move |_| Box::pin(std::future::ready(Ok(Some(issue_clone.clone())))));

        let handler = GetIssueHandler::new(
            std::sync::Arc::new(project_store),
            std::sync::Arc::new(issue_store),
        );
        let cmd = GetIssue {
            project: ProjectIdentifier::Slug(org_id, slug),
            issue_id,
        };
        let ctx = RequestContext::test();

        // act
        let result = handler.handle(cmd, &ctx).await;

        // assert
        assert!(matches!(result, Err(crate::error::ApplicationError::NotFound)));
    }
}
