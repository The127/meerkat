use async_trait::async_trait;

use meerkat_domain::models::permission::ProjectPermission;
use meerkat_domain::models::project::ProjectIdentifier;

use crate::behaviors::authorization::project_extensions;
use crate::context::RequestContext;
use crate::error::ApplicationError;
use mediator_rs::{Extensions, Request, Handler};
use crate::ports::project_read_store::{ProjectReadModel, ProjectReadStore};

pub struct GetProject {
    pub project: ProjectIdentifier,
}

impl Request for GetProject {
    type Output = ProjectReadModel;

    fn extensions(&self) -> Extensions {
        project_extensions(
            "GetProject",
            vec![ProjectPermission::ProjectRead.into()],
            self.project.clone(),
        )
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
        let ProjectIdentifier::Slug(ref org_id, ref slug) = cmd.project else {
            return Err(ApplicationError::NotFound);
        };
        self.project_read_store
            .find_by_slug(org_id, slug)
            .await?
            .ok_or(ApplicationError::NotFound)
    }
}
