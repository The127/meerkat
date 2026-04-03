use async_trait::async_trait;

use meerkat_domain::models::project::ProjectIdentifier;

use meerkat_domain::models::permission::ProjectPermission;

use crate::behaviors::authorization::{ProjectContext, RequestName, RequiredPermissions};
use crate::context::RequestContext;
use crate::error::ApplicationError;
use crate::extensions::Extensions;
use crate::mediator::{Request, Handler};

pub struct RenameProject {
    pub identifier: ProjectIdentifier,
    pub name: String,
}

impl Request for RenameProject {
    type Output = ();

    fn extensions(&self) -> Extensions {
        let mut ext = Extensions::new();
        ext.insert(RequestName("RenameProject".to_string()));
        ext.insert(RequiredPermissions(vec![ProjectPermission::ProjectWrite.into()]));
        ext.insert(ProjectContext(self.identifier.clone()));
        ext
    }
}

pub struct RenameProjectHandler;

#[async_trait]
impl Handler<RenameProject, ApplicationError, RequestContext> for RenameProjectHandler {
    async fn handle(
        &self,
        cmd: RenameProject,
        ctx: &RequestContext,
    ) -> Result<(), ApplicationError> {
        let uow = ctx.uow().await;

        let mut project = uow.projects().find(&cmd.identifier).await?;

        project.update_name(cmd.name)?;

        uow.projects().save(project);

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

    use super::{RenameProject, RenameProjectHandler};

    #[tokio::test]
    async fn given_valid_name_then_finds_project_and_saves_renamed() {
        // arrange
        let (project, _clock) = test_project();
        let project_id = project.id().clone();
        let expected_id = project_id.clone();

        let mut repo = MockProjectRepository::new();
        repo.expect_find()
            .times(1)
            .withf(move |identifier| matches!(identifier, ProjectIdentifier::Id(id) if *id == expected_id))
            .returning(move |_| Box::pin(std::future::ready(Ok(project.clone()))));
        repo.expect_save()
            .times(1)
            .withf(|project| project.name() == "New Name")
            .returning(|_| ());

        let ctx = RequestContext::test()
            .with_scoped_uow(Box::new(MockUnitOfWork::new().with_project_repo(repo)));

        let handler = RenameProjectHandler;
        let cmd = RenameProject {
            identifier: ProjectIdentifier::Id(project_id),
            name: "New Name".to_string(),
        };

        // act
        let result = handler.handle(cmd, &ctx).await;

        // assert
        assert!(result.is_ok());
    }
}
