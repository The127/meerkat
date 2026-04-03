use std::sync::Arc;

use async_trait::async_trait;

use meerkat_domain::models::organization::OrganizationId;
use meerkat_domain::models::permission::OrgPermission;

use crate::behaviors::authorization::{RequestName, RequiredPermissions};
use crate::context::RequestContext;
use crate::error::ApplicationError;
use crate::extensions::Extensions;
use crate::mediator::{Request, Handler};
use crate::ports::member_read_store::{MemberReadModel, MemberReadStore};

pub struct ListMembers {
    pub org_id: OrganizationId,
}

impl Request for ListMembers {
    type Output = Vec<MemberReadModel>;

    fn extensions(&self) -> Extensions {
        let mut ext = Extensions::new();
        ext.insert(RequestName("ListMembers".to_string()));
        ext.insert(RequiredPermissions(vec![OrgPermission::OrgManageMembers.into()]));
        ext
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
    ) -> Result<Vec<MemberReadModel>, ApplicationError> {
        self.member_read_store.list_by_org(&cmd.org_id).await
    }
}
