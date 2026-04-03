use std::sync::Arc;

use async_trait::async_trait;

use meerkat_domain::models::member::MemberId;
use meerkat_domain::models::permission::OrgPermission;

use crate::behaviors::authorization::{RequestName, RequiredPermissions};
use crate::context::RequestContext;
use crate::error::ApplicationError;
use crate::extensions::Extensions;
use crate::mediator::{Request, Handler};
use crate::ports::project_member_read_store::{MemberProjectReadModel, ProjectMemberReadStore};

pub struct ListMemberProjects {
    pub member_id: MemberId,
}

impl Request for ListMemberProjects {
    type Output = Vec<MemberProjectReadModel>;

    fn extensions(&self) -> Extensions {
        let mut ext = Extensions::new();
        ext.insert(RequestName("ListMemberProjects".to_string()));
        ext.insert(RequiredPermissions(vec![OrgPermission::OrgManageMembers.into()]));
        ext
    }
}

pub struct ListMemberProjectsHandler {
    project_member_read_store: Arc<dyn ProjectMemberReadStore>,
}

impl ListMemberProjectsHandler {
    pub fn new(project_member_read_store: Arc<dyn ProjectMemberReadStore>) -> Self {
        Self { project_member_read_store }
    }
}

#[async_trait]
impl Handler<ListMemberProjects, ApplicationError, RequestContext> for ListMemberProjectsHandler {
    async fn handle(
        &self,
        cmd: ListMemberProjects,
        _ctx: &RequestContext,
    ) -> Result<Vec<MemberProjectReadModel>, ApplicationError> {
        self.project_member_read_store.list_by_member(&cmd.member_id).await
    }
}
