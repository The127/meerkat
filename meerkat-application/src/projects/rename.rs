use async_trait::async_trait;

use meerkat_domain::models::project::ProjectId;

use crate::context::RequestContext;
use crate::error::ApplicationError;
use crate::mediator::{Command, Handler};

pub struct RenameProject {
    pub project_id: ProjectId,
    pub name: String,
}

impl Command for RenameProject {
    type Output = ();
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

        let mut project = uow.projects().find_by_id(&cmd.project_id).await?;

        project.update_name(cmd.name)?;

        uow.projects().save(project);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use meerkat_domain::testing::test_project;

    use crate::context::RequestContext;
    use crate::mediator::Handler;
    use crate::ports::project_repository::MockProjectRepository;
    use crate::ports::unit_of_work::MockUnitOfWork;

    use super::{RenameProject, RenameProjectHandler};

    #[tokio::test]
    async fn given_valid_name_then_fetches_by_id_and_saves_renamed_project() {
        // arrange
        let (project, _clock) = test_project();
        let project_id = project.id().clone();
        let expected_id = project_id.clone();

        let mut repo = MockProjectRepository::new();
        repo.expect_find_by_id()
            .times(1)
            .withf(move |id| *id == expected_id)
            .returning(move |_| Box::pin(std::future::ready(Ok(project.clone()))));
        repo.expect_save()
            .times(1)
            .withf(|project| project.name() == "New Name")
            .returning(|_| ());

        let ctx = RequestContext::test()
            .with_scoped_uow(Box::new(MockUnitOfWork::new().with_project_repo(repo)));

        let handler = RenameProjectHandler;
        let cmd = RenameProject {
            project_id,
            name: "New Name".to_string(),
        };

        // act
        let result = handler.handle(cmd, &ctx).await;

        // assert
        assert!(result.is_ok());
    }
}
