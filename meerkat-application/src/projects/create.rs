use async_trait::async_trait;

use meerkat_domain::models::organization::OrganizationId;
use meerkat_domain::models::project::{Project, ProjectId, ProjectSlug};

use crate::context::RequestContext;
use crate::error::ApplicationError;
use crate::mediator::{Command, Handler};

pub struct CreateProject {
    pub organization_id: OrganizationId,
    pub name: String,
    pub slug: ProjectSlug,
}

impl Command for CreateProject {
    type Output = ProjectId;
}

pub struct CreateProjectHandler;

#[async_trait]
impl Handler<CreateProject, ApplicationError, RequestContext> for CreateProjectHandler {
    async fn handle(
        &self,
        cmd: CreateProject,
        ctx: &RequestContext,
    ) -> Result<ProjectId, ApplicationError> {
        let project = Project::new(cmd.organization_id, cmd.name, cmd.slug, ctx.clock())
            .map_err(|e| ApplicationError::Validation(e.to_string()))?;

        let id = project.id().clone();

        ctx.with_uow(|uow| uow.projects().add(project));

        Ok(id)
    }
}

#[cfg(test)]
mod tests {
    use meerkat_domain::models::organization::OrganizationId;
    use meerkat_domain::models::project::ProjectSlug;

    use crate::context::RequestContext;
    use crate::error::ApplicationError;
    use crate::mediator::Handler;
    use crate::ports::project_repository::MockProjectRepository;
    use crate::ports::unit_of_work::MockUnitOfWork;

    use super::{CreateProject, CreateProjectHandler};

    #[tokio::test]
    async fn given_valid_input_when_creating_project_it_should_return_an_id() {
        // arrange
        let mut repo = MockProjectRepository::new();
        repo.expect_add().times(1).returning(|_| ());

        let ctx = RequestContext::test()
            .with_scoped_uow(Box::new(MockUnitOfWork::new().with_project_repo(repo)));

        let handler = CreateProjectHandler;
        let cmd = CreateProject {
            organization_id: OrganizationId::new(),
            name: "My Project".to_string(),
            slug: ProjectSlug::new("my-project").unwrap(),
        };

        // act
        let result = handler.handle(cmd, &ctx).await;

        // assert
        assert!(result.is_ok());
        assert!(!result.unwrap().as_uuid().is_nil());
    }

    #[tokio::test]
    async fn given_empty_name_when_creating_project_it_should_return_validation_error() {
        // arrange
        let ctx = RequestContext::test();
        let handler = CreateProjectHandler;
        let cmd = CreateProject {
            organization_id: OrganizationId::new(),
            name: "  ".to_string(),
            slug: ProjectSlug::new("some-slug").unwrap(),
        };

        // act
        let result = handler.handle(cmd, &ctx).await;

        // assert
        match result {
            Err(ApplicationError::Validation(_)) => (),
            other => panic!("Expected Validation error, got {:?}", other),
        }
    }
}
