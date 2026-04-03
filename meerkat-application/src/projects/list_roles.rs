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
use crate::ports::project_read_store::ProjectReadStore;
use crate::ports::project_role_read_store::{ProjectRoleReadModel, ProjectRoleReadStore};

pub struct ListProjectRoles {
    pub org_id: OrganizationId,
    pub slug: ProjectSlug,
}

impl Request for ListProjectRoles {
    type Output = Vec<ProjectRoleReadModel>;

    fn extensions(&self) -> Extensions {
        let mut ext = Extensions::new();
        ext.insert(RequestName("ListProjectRoles".to_string()));
        ext.insert(RequiredPermissions(vec![ProjectPermission::ProjectRead.into()]));
        ext.insert(ProjectContext(ProjectIdentifier::Slug(self.org_id.clone(), self.slug.clone())));
        ext
    }
}

pub struct ListProjectRolesHandler {
    project_read_store: Arc<dyn ProjectReadStore>,
    project_role_read_store: Arc<dyn ProjectRoleReadStore>,
}

impl ListProjectRolesHandler {
    pub fn new(
        project_read_store: Arc<dyn ProjectReadStore>,
        project_role_read_store: Arc<dyn ProjectRoleReadStore>,
    ) -> Self {
        Self { project_read_store, project_role_read_store }
    }
}

#[async_trait]
impl Handler<ListProjectRoles, ApplicationError, RequestContext> for ListProjectRolesHandler {
    async fn handle(
        &self,
        cmd: ListProjectRoles,
        _ctx: &RequestContext,
    ) -> Result<Vec<ProjectRoleReadModel>, ApplicationError> {
        let project = self.project_read_store
            .find_by_slug(&cmd.org_id, &cmd.slug)
            .await?
            .ok_or(ApplicationError::NotFound)?;

        self.project_role_read_store.list_by_project(&project.id).await
    }
}
