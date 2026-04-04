use std::sync::Arc;

use async_trait::async_trait;

use meerkat_domain::models::permission::ProjectPermission;
use meerkat_domain::models::project::ProjectIdentifier;

use crate::behaviors::authorization::project_extensions;
use crate::context::RequestContext;
use crate::error::ApplicationError;
use crate::extensions::Extensions;
use crate::mediator::{Request, Handler};
use crate::ports::project_key_read_store::{ProjectKeyReadModel, ProjectKeyReadStore};
use crate::ports::project_read_store::{PagedResult, ProjectReadStore};
use crate::search::SearchFilter;

pub struct ListProjectKeys {
    pub project: ProjectIdentifier,
    pub search: Option<SearchFilter>,
    pub status: Option<String>,
    pub limit: i64,
    pub offset: i64,
}

impl Request for ListProjectKeys {
    type Output = PagedResult<ProjectKeyReadModel>;

    fn extensions(&self) -> Extensions {
        project_extensions(
            "ListProjectKeys",
            vec![ProjectPermission::ProjectRead.into()],
            self.project.clone(),
        )
    }
}

pub struct ListProjectKeysHandler {
    project_read_store: Arc<dyn ProjectReadStore>,
    project_key_read_store: Arc<dyn ProjectKeyReadStore>,
}

impl ListProjectKeysHandler {
    pub fn new(
        project_read_store: Arc<dyn ProjectReadStore>,
        project_key_read_store: Arc<dyn ProjectKeyReadStore>,
    ) -> Self {
        Self { project_read_store, project_key_read_store }
    }
}

#[async_trait]
impl Handler<ListProjectKeys, ApplicationError, RequestContext> for ListProjectKeysHandler {
    async fn handle(
        &self,
        cmd: ListProjectKeys,
        _ctx: &RequestContext,
    ) -> Result<PagedResult<ProjectKeyReadModel>, ApplicationError> {
        let ProjectIdentifier::Slug(ref org_id, ref slug) = cmd.project else {
            return Err(ApplicationError::NotFound);
        };
        let project = self.project_read_store
            .find_by_slug(org_id, slug)
            .await?
            .ok_or(ApplicationError::NotFound)?;

        self.project_key_read_store
            .list_by_project(&project.id, cmd.search.as_ref(), cmd.status.as_deref(), cmd.limit, cmd.offset)
            .await
    }
}
