use async_trait::async_trait;

use meerkat_domain::models::organization::OrganizationId;
use meerkat_domain::models::project::{Project, ProjectId, ProjectSlug};

use meerkat_domain::models::permission::ProjectPermission;

use crate::behaviors::authorization::RequiredPermissions;
use crate::context::RequestContext;
use crate::error::ApplicationError;
use crate::extensions::Extensions;
use crate::mediator::{Command, Handler};

pub struct CreateProject {
    pub organization_id: OrganizationId,
    pub name: String,
    pub slug: ProjectSlug,
}

impl Command for CreateProject {
    type Output = ProjectId;

    fn extensions(&self) -> Extensions {
        let mut ext = Extensions::new();
        ext.insert(RequiredPermissions(vec![ProjectPermission::ProjectWrite.into()]));
        ext
    }
}

pub struct CreateProjectHandler;

#[async_trait]
impl Handler<CreateProject, ApplicationError, RequestContext> for CreateProjectHandler {
    async fn handle(
        &self,
        cmd: CreateProject,
        ctx: &RequestContext,
    ) -> Result<ProjectId, ApplicationError> {
        let project = Project::new(cmd.organization_id, cmd.name, cmd.slug, ctx.clock())?;

        let id = project.id().clone();

        ctx.uow().await.projects().add(project);

        Ok(id)
    }
}

#[cfg(test)]
mod tests {
    use meerkat_domain::models::organization::OrganizationId;
    use meerkat_domain::models::project::ProjectSlug;

    use crate::context::RequestContext;
    use crate::mediator::Handler;
    use crate::ports::project_repository::MockProjectRepository;
    use crate::ports::unit_of_work::MockUnitOfWork;

    use super::{CreateProject, CreateProjectHandler};

    #[tokio::test]
    async fn given_valid_input_then_adds_project_with_correct_fields_and_returns_id() {
        // arrange
        let org_id = OrganizationId::new();
        let expected_org_id = org_id.clone();

        let mut repo = MockProjectRepository::new();
        repo.expect_add()
            .times(1)
            .withf(move |project| {
                *project.organization_id() == expected_org_id
                    && project.name() == "My Project"
                    && project.slug().as_str() == "my-project"
            })
            .returning(|_| ());

        let ctx = RequestContext::test()
            .with_scoped_uow(Box::new(MockUnitOfWork::new().with_project_repo(repo)));

        let handler = CreateProjectHandler;
        let cmd = CreateProject {
            organization_id: org_id,
            name: "My Project".to_string(),
            slug: ProjectSlug::new("my-project").unwrap(),
        };

        // act
        let result = handler.handle(cmd, &ctx).await;

        // assert
        assert!(result.is_ok());
        assert!(!result.unwrap().as_uuid().is_nil());
    }
}
