use std::sync::Arc;

use async_trait::async_trait;

use meerkat_domain::models::permission::ProjectPermission;
use meerkat_domain::models::project::ProjectIdentifier;

use crate::behaviors::authorization::project_extensions;
use crate::context::RequestContext;
use crate::error::ApplicationError;
use mediator_rs::{Extensions, Request, Handler};
use crate::ports::project_member_read_store::{ProjectMemberReadModel, ProjectMemberReadStore};
use crate::ports::project_read_store::ProjectReadStore;

pub struct ListProjectMembers {
    pub project: ProjectIdentifier,
}

impl Request for ListProjectMembers {
    type Output = Vec<ProjectMemberReadModel>;

    fn extensions(&self) -> Extensions {
        project_extensions(
            "ListProjectMembers",
            vec![ProjectPermission::ProjectRead.into()],
            self.project.clone(),
        )
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
        let ProjectIdentifier::Slug(ref org_id, ref slug) = cmd.project else {
            return Err(ApplicationError::NotFound);
        };
        let project = self.project_read_store
            .find_by_slug(org_id, slug)
            .await?
            .ok_or(ApplicationError::NotFound)?;

        self.project_member_read_store.list_by_project(&project.id).await
    }
}
