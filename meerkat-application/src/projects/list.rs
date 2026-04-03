use async_trait::async_trait;

use meerkat_domain::models::organization::OrganizationId;
use meerkat_domain::models::permission::ProjectPermission;

use crate::behaviors::authorization::{RequestName, RequiredPermissions};
use crate::context::RequestContext;
use crate::error::ApplicationError;
use crate::extensions::Extensions;
use crate::mediator::{Request, Handler};
use crate::ports::project_read_store::{PagedResult, ProjectReadModel, ProjectReadStore};
use crate::search::SearchFilter;

pub struct ListProjects {
    pub org_id: OrganizationId,
    pub search: Option<SearchFilter>,
    pub limit: i64,
    pub offset: i64,
}

impl Request for ListProjects {
    type Output = PagedResult<ProjectReadModel>;

    fn extensions(&self) -> Extensions {
        let mut ext = Extensions::new();
        ext.insert(RequestName("ListProjects".to_string()));
        ext.insert(RequiredPermissions(vec![ProjectPermission::ProjectRead.into()]));
        ext
    }
}

pub struct ListProjectsHandler {
    project_read_store: std::sync::Arc<dyn ProjectReadStore>,
}

impl ListProjectsHandler {
    pub fn new(project_read_store: std::sync::Arc<dyn ProjectReadStore>) -> Self {
        Self { project_read_store }
    }
}

#[async_trait]
impl Handler<ListProjects, ApplicationError, RequestContext> for ListProjectsHandler {
    async fn handle(
        &self,
        cmd: ListProjects,
        _ctx: &RequestContext,
    ) -> Result<PagedResult<ProjectReadModel>, ApplicationError> {
        self.project_read_store
            .list_by_org(&cmd.org_id, cmd.search.as_ref(), cmd.limit, cmd.offset)
            .await
    }
}
