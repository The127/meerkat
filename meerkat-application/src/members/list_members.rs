use std::sync::Arc;

use async_trait::async_trait;

use meerkat_domain::models::permission::OrgPermission;

use crate::behaviors::authorization::org_extensions;
use crate::context::RequestContext;
use crate::error::ApplicationError;
use mediator_rs::{Extensions, Request, Handler};
use crate::ports::member_read_store::{ListMembersQuery, MemberReadModel, MemberReadStore};
use crate::ports::project_read_store::PagedResult;

pub struct ListMembers {
    pub query: ListMembersQuery,
}

impl Request for ListMembers {
    type Output = PagedResult<MemberReadModel>;

    fn extensions(&self) -> Extensions {
        org_extensions("ListMembers", vec![OrgPermission::OrgManageMembers.into()])
    }
}

pub struct ListMembersHandler {
    member_read_store: Arc<dyn MemberReadStore>,
}

impl ListMembersHandler {
    pub fn new(member_read_store: Arc<dyn MemberReadStore>) -> Self {
        Self { member_read_store }
    }
}

#[async_trait]
impl Handler<ListMembers, ApplicationError, RequestContext> for ListMembersHandler {
    async fn handle(
        &self,
        cmd: ListMembers,
        _ctx: &RequestContext,
    ) -> Result<PagedResult<MemberReadModel>, ApplicationError> {
        self.member_read_store
            .list_by_org(&cmd.query)
            .await
    }
}
