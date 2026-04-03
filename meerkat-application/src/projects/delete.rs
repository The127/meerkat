use async_trait::async_trait;

use meerkat_domain::models::project::ProjectIdentifier;

use meerkat_domain::models::permission::ProjectPermission;

use crate::behaviors::authorization::{RequestName, RequiredPermissions};
use crate::context::RequestContext;
use crate::error::ApplicationError;
use crate::extensions::Extensions;
use crate::mediator::{Request, Handler};

pub struct DeleteProject {
    pub identifier: ProjectIdentifier,
}

impl Request for DeleteProject {
    type Output = ();

    fn extensions(&self) -> Extensions {
        let mut ext = Extensions::new();
        ext.insert(RequestName("DeleteProject".to_string()));
        ext.insert(RequiredPermissions(vec![ProjectPermission::ProjectDelete.into()]));
        ext
    }
}

pub struct DeleteProjectHandler;

#[async_trait]
impl Handler<DeleteProject, ApplicationError, RequestContext> for DeleteProjectHandler {
    async fn handle(
        &self,
        cmd: DeleteProject,
        ctx: &RequestContext,
    ) -> Result<(), ApplicationError> {
        let uow = ctx.uow().await;

        let project = uow.projects().find(&cmd.identifier).await?;

        uow.projects().delete(project.id().clone());

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use meerkat_domain::models::project::ProjectIdentifier;
    use meerkat_domain::testing::test_project;

    use crate::context::RequestContext;
    use crate::mediator::Handler;
    use crate::ports::project_repository::MockProjectRepository;
    use crate::ports::unit_of_work::MockUnitOfWork;

    use super::{DeleteProject, DeleteProjectHandler};

    #[tokio::test]
    async fn given_existing_project_then_finds_and_deletes() {
        // arrange
        let (project, _clock) = test_project();
        let project_id = project.id().clone();
        let expected_id = project_id.clone();
        let expected_delete_id = project_id.clone();

        let mut repo = MockProjectRepository::new();
        repo.expect_find()
            .times(1)
            .withf(move |identifier| matches!(identifier, ProjectIdentifier::Id(id) if *id == expected_id))
            .returning(move |_| Box::pin(std::future::ready(Ok(project.clone()))));
        repo.expect_delete()
            .times(1)
            .withf(move |id| *id == expected_delete_id)
            .returning(|_| ());

        let ctx = RequestContext::test()
            .with_scoped_uow(Box::new(MockUnitOfWork::new().with_project_repo(repo)));

        let handler = DeleteProjectHandler;
        let cmd = DeleteProject {
            identifier: ProjectIdentifier::Id(project_id),
        };

        // act
        let result = handler.handle(cmd, &ctx).await;

        // assert
        assert!(result.is_ok());
    }
}
