use std::sync::Arc;

use async_trait::async_trait;

use meerkat_domain::models::issue::IssueStatus;
use meerkat_domain::models::permission::ProjectPermission;
use meerkat_domain::models::project::ProjectIdentifier;

use crate::behaviors::authorization::project_extensions;
use crate::context::RequestContext;
use crate::error::ApplicationError;
use crate::extensions::Extensions;
use crate::mediator::{Request, Handler};
use crate::ports::issue_read_store::{IssueReadModel, IssueReadStore};
use crate::ports::project_read_store::{PagedResult, ProjectReadStore};
use crate::search::SearchFilter;

pub struct ListIssues {
    pub project: ProjectIdentifier,
    pub statuses: Vec<IssueStatus>,
    pub search: Option<SearchFilter>,
    pub limit: i64,
    pub offset: i64,
}

impl Request for ListIssues {
    type Output = PagedResult<IssueReadModel>;

    fn extensions(&self) -> Extensions {
        project_extensions(
            "ListIssues",
            vec![ProjectPermission::ProjectRead.into()],
            self.project.clone(),
        )
    }
}

pub struct ListIssuesHandler {
    project_read_store: Arc<dyn ProjectReadStore>,
    issue_read_store: Arc<dyn IssueReadStore>,
}

impl ListIssuesHandler {
    pub fn new(
        project_read_store: Arc<dyn ProjectReadStore>,
        issue_read_store: Arc<dyn IssueReadStore>,
    ) -> Self {
        Self { project_read_store, issue_read_store }
    }
}

#[async_trait]
impl Handler<ListIssues, ApplicationError, RequestContext> for ListIssuesHandler {
    async fn handle(
        &self,
        cmd: ListIssues,
        _ctx: &RequestContext,
    ) -> Result<PagedResult<IssueReadModel>, ApplicationError> {
        let ProjectIdentifier::Slug(ref org_id, ref slug) = cmd.project else {
            return Err(ApplicationError::NotFound);
        };
        let project = self.project_read_store
            .find_by_slug(org_id, slug)
            .await?
            .ok_or(ApplicationError::NotFound)?;

        self.issue_read_store
            .list_by_project(&project.id, &cmd.statuses, cmd.search.as_ref(), cmd.limit, cmd.offset)
            .await
    }
}
