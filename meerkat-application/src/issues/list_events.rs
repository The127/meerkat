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
use crate::ports::event_read_store::{EventReadModel, EventReadStore};
use crate::ports::issue_read_store::IssueReadStore;
use crate::ports::project_read_store::{PagedResult, ProjectReadStore};

pub struct ListIssueEvents {
    pub project: ProjectIdentifier,
    pub issue_id: IssueId,
    pub limit: i64,
    pub offset: i64,
}

impl Request for ListIssueEvents {
    type Output = PagedResult<EventReadModel>;

    fn extensions(&self) -> Extensions {
        project_extensions(
            "ListIssueEvents",
            vec![ProjectPermission::ProjectRead.into()],
            self.project.clone(),
        )
    }
}

pub struct ListIssueEventsHandler {
    project_read_store: Arc<dyn ProjectReadStore>,
    issue_read_store: Arc<dyn IssueReadStore>,
    event_read_store: Arc<dyn EventReadStore>,
}

impl ListIssueEventsHandler {
    pub fn new(
        project_read_store: Arc<dyn ProjectReadStore>,
        issue_read_store: Arc<dyn IssueReadStore>,
        event_read_store: Arc<dyn EventReadStore>,
    ) -> Self {
        Self { project_read_store, issue_read_store, event_read_store }
    }
}

#[async_trait]
impl Handler<ListIssueEvents, ApplicationError, RequestContext> for ListIssueEventsHandler {
    async fn handle(
        &self,
        cmd: ListIssueEvents,
        _ctx: &RequestContext,
    ) -> Result<PagedResult<EventReadModel>, ApplicationError> {
        let ProjectIdentifier::Slug(ref org_id, ref slug) = cmd.project else {
            return Err(ApplicationError::NotFound);
        };
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

        self.event_read_store
            .list_by_issue(&cmd.issue_id, cmd.limit, cmd.offset)
            .await
    }
}

#[cfg(test)]
mod tests {
    use meerkat_domain::models::event::EventId;
    use meerkat_domain::models::issue::IssueId;
    use meerkat_domain::models::organization::OrganizationId;
    use meerkat_domain::models::project::{ProjectId, ProjectIdentifier, ProjectSlug};

    use crate::context::RequestContext;
    use crate::mediator::Handler;
    use crate::ports::event_read_store::{EventReadModel, MockEventReadStore};
    use crate::ports::issue_read_store::{IssueReadModel, MockIssueReadStore};
    use crate::ports::project_read_store::{MockProjectReadStore, PagedResult, ProjectReadModel};

    use super::{ListIssueEvents, ListIssueEventsHandler};

    fn test_event_read_model(issue_id: IssueId) -> EventReadModel {
        EventReadModel {
            id: EventId::new(),
            project_id: ProjectId::new(),
            issue_id,
            fingerprint_hash: "abc123".to_string(),
            message: "Something broke".to_string(),
            level: "error".to_string(),
            platform: "javascript".to_string(),
            timestamp: chrono::Utc::now(),
            server_name: Some("web-1".to_string()),
            environment: Some("production".to_string()),
            release: Some("v1.0.0".to_string()),
            exception_type: Some("TypeError".to_string()),
            exception_value: Some("x is not defined".to_string()),
            tags: vec![("browser".to_string(), "Chrome".to_string())],
            extra: serde_json::json!({}),
        }
    }

    #[tokio::test]
    async fn given_existing_issue_then_returns_events() {
        // arrange
        let org_id = OrganizationId::new();
        let project_id = ProjectId::new();
        let slug = ProjectSlug::new("test-project").unwrap();
        let issue_id = IssueId::new();

        let mut project_store = MockProjectReadStore::new();
        let org_id_clone = org_id.clone();
        let project_id_clone = project_id.clone();
        project_store.expect_find_by_slug()
            .times(1)
            .withf(move |o, s| *o == org_id_clone && s.as_str() == "test-project")
            .returning(move |_, _| {
                let pid = project_id_clone.clone();
                Box::pin(std::future::ready(Ok(Some(ProjectReadModel {
                    id: pid,
                    organization_id: OrganizationId::new(),
                    name: "Test Project".to_string(),
                    slug: ProjectSlug::new("test-project").unwrap(),
                    created_at: chrono::Utc::now(),
                    updated_at: chrono::Utc::now(),
                }))))
            });

        let mut issue_store = MockIssueReadStore::new();
        let issue_id_clone = issue_id.clone();
        let project_id_clone2 = project_id.clone();
        issue_store.expect_find_by_id()
            .times(1)
            .withf(move |id| *id == issue_id_clone)
            .returning(move |_| {
                let pid = project_id_clone2.clone();
                Box::pin(std::future::ready(Ok(Some(IssueReadModel {
                    id: IssueId::new(),
                    project_id: pid,
                    title: "Test".to_string(),
                    fingerprint_hash: "abc".to_string(),
                    status: "unresolved".to_string(),
                    level: "error".to_string(),
                    event_count: 1,
                    first_seen: chrono::Utc::now(),
                    last_seen: chrono::Utc::now(),
                }))))
            });

        let mut event_store = MockEventReadStore::new();
        let issue_id_clone2 = issue_id.clone();
        event_store.expect_list_by_issue()
            .times(1)
            .withf(move |id, l, o| *id == issue_id_clone2 && *l == 20 && *o == 0)
            .returning(move |_, _, _| {
                let e = test_event_read_model(IssueId::new());
                Box::pin(std::future::ready(Ok(PagedResult { items: vec![e], total: 1 })))
            });

        let handler = ListIssueEventsHandler::new(
            std::sync::Arc::new(project_store),
            std::sync::Arc::new(issue_store),
            std::sync::Arc::new(event_store),
        );
        let cmd = ListIssueEvents {
            project: ProjectIdentifier::Slug(org_id, slug),
            issue_id,
            limit: 20,
            offset: 0,
        };
        let ctx = RequestContext::test();

        // act
        let result = handler.handle(cmd, &ctx).await;

        // assert
        let paged = result.unwrap();
        assert_eq!(paged.total, 1);
        assert_eq!(paged.items.len(), 1);
    }
}
