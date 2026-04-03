use std::sync::Arc;

use async_trait::async_trait;

use meerkat_domain::models::organization::OrganizationId;
use meerkat_domain::models::permission::ProjectPermission;
use meerkat_domain::models::project::{ProjectIdentifier, ProjectSlug};

use crate::behaviors::authorization::{ProjectContext, RequestName, RequiredPermissions};
use crate::context::RequestContext;
use crate::error::ApplicationError;
use crate::extensions::Extensions;
use crate::mediator::{Request, Handler};
use crate::ports::project_member_read_store::{ProjectMemberReadModel, ProjectMemberReadStore};
use crate::ports::project_read_store::ProjectReadStore;

pub struct ListProjectMembers {
    pub org_id: OrganizationId,
    pub slug: ProjectSlug,
}

impl Request for ListProjectMembers {
    type Output = Vec<ProjectMemberReadModel>;

    fn extensions(&self) -> Extensions {
        let mut ext = Extensions::new();
        ext.insert(RequestName("ListProjectMembers".to_string()));
        ext.insert(RequiredPermissions(vec![ProjectPermission::ProjectRead.into()]));
        ext.insert(ProjectContext(ProjectIdentifier::Slug(self.org_id.clone(), self.slug.clone())));
        ext
    }
}

pub struct ListProjectMembersHandler {
    project_read_store: Arc<dyn ProjectReadStore>,
    project_member_read_store: Arc<dyn ProjectMemberReadStore>,
}

impl ListProjectMembersHandler {
    pub fn new(
        project_read_store: Arc<dyn ProjectReadStore>,
        project_member_read_store: Arc<dyn ProjectMemberReadStore>,
    ) -> Self {
        Self { project_read_store, project_member_read_store }
    }
}

#[async_trait]
impl Handler<ListProjectMembers, ApplicationError, RequestContext> for ListProjectMembersHandler {
    async fn handle(
        &self,
        cmd: ListProjectMembers,
        _ctx: &RequestContext,
    ) -> Result<Vec<ProjectMemberReadModel>, ApplicationError> {
        let project = self.project_read_store
            .find_by_slug(&cmd.org_id, &cmd.slug)
            .await?
            .ok_or(ApplicationError::NotFound)?;

        self.project_member_read_store.list_by_project(&project.id).await
    }
}
