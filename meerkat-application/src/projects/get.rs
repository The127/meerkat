use async_trait::async_trait;

use meerkat_domain::models::organization::OrganizationId;
use meerkat_domain::models::permission::ProjectPermission;
use meerkat_domain::models::project::{ProjectIdentifier, ProjectSlug};

use crate::behaviors::authorization::{ProjectContext, RequestName, RequiredPermissions};
use crate::context::RequestContext;
use crate::error::ApplicationError;
use crate::extensions::Extensions;
use crate::mediator::{Request, Handler};
use crate::ports::project_read_store::{ProjectReadModel, ProjectReadStore};

pub struct GetProject {
    pub org_id: OrganizationId,
    pub slug: ProjectSlug,
}

impl Request for GetProject {
    type Output = ProjectReadModel;

    fn extensions(&self) -> Extensions {
        let mut ext = Extensions::new();
        ext.insert(RequestName("GetProject".to_string()));
        ext.insert(RequiredPermissions(vec![ProjectPermission::ProjectRead.into()]));
        ext.insert(ProjectContext(ProjectIdentifier::Slug(self.org_id.clone(), self.slug.clone())));
        ext
    }
}

pub struct GetProjectHandler {
    project_read_store: std::sync::Arc<dyn ProjectReadStore>,
}

impl GetProjectHandler {
    pub fn new(project_read_store: std::sync::Arc<dyn ProjectReadStore>) -> Self {
        Self { project_read_store }
    }
}

#[async_trait]
impl Handler<GetProject, ApplicationError, RequestContext> for GetProjectHandler {
    async fn handle(
        &self,
        cmd: GetProject,
        _ctx: &RequestContext,
    ) -> Result<ProjectReadModel, ApplicationError> {
        self.project_read_store
            .find_by_slug(&cmd.org_id, &cmd.slug)
            .await?
            .ok_or(ApplicationError::NotFound)
    }
}
