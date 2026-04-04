use async_trait::async_trait;

use meerkat_domain::models::permission::ProjectPermission;
use meerkat_domain::models::project::ProjectIdentifier;
use meerkat_domain::models::project_key::{ProjectKey, ProjectKeyId};

use crate::behaviors::authorization::project_extensions;
use crate::context::RequestContext;
use crate::error::ApplicationError;
use crate::extensions::Extensions;
use crate::mediator::{Request, Handler};

pub struct CreateProjectKey {
    pub project: ProjectIdentifier,
    pub label: String,
}

impl Request for CreateProjectKey {
    type Output = ProjectKeyId;

    fn extensions(&self) -> Extensions {
        project_extensions(
            "CreateProjectKey",
            vec![ProjectPermission::ProjectManageKeys.into()],
            self.project.clone(),
        )
    }
}

pub struct CreateProjectKeyHandler;

#[async_trait]
impl Handler<CreateProjectKey, ApplicationError, RequestContext> for CreateProjectKeyHandler {
    async fn handle(
        &self,
        cmd: CreateProjectKey,
        ctx: &RequestContext,
    ) -> Result<ProjectKeyId, ApplicationError> {
        let uow = ctx.uow().await;

        let project = uow.projects()
            .find(&cmd.project)
            .await?;

        let key = ProjectKey::generate(project.id().clone(), cmd.label)?;
        let key_id = key.id().clone();
        uow.project_keys().add(key);

        Ok(key_id)
    }
}

#[cfg(test)]
mod tests {
    use meerkat_domain::models::organization::OrganizationId;
    use meerkat_domain::models::project::{ProjectIdentifier, ProjectSlug};
    use meerkat_domain::testing::test_project;

    use crate::context::RequestContext;
    use crate::mediator::Handler;
    use crate::ports::project_key_repository::MockProjectKeyRepository;
    use crate::ports::project_repository::MockProjectRepository;
    use crate::ports::unit_of_work::MockUnitOfWork;

    use super::{CreateProjectKey, CreateProjectKeyHandler};

    #[tokio::test]
    async fn given_valid_input_then_generates_key_and_adds_to_uow() {
        // arrange
        let project = test_project();
        let project_id = project.id().clone();
        let expected_project_id = project_id.clone();

        let mut project_repo = MockProjectRepository::new();
        project_repo.expect_find()
            .times(1)
            .returning(move |_| Box::pin(std::future::ready(Ok(project.clone()))));

        let mut key_repo = MockProjectKeyRepository::new();
        key_repo.expect_add()
            .times(1)
            .withf(move |key| {
                *key.project_id() == expected_project_id
                    && key.label() == "Production"
            })
            .returning(|_| ());

        let ctx = RequestContext::test()
            .with_scoped_uow(Box::new(
                MockUnitOfWork::new()
                    .with_project_repo(project_repo)
                    .with_project_key_repo(key_repo),
            ));

        let handler = CreateProjectKeyHandler;
        let cmd = CreateProjectKey {
            project: ProjectIdentifier::Slug(OrganizationId::new(), ProjectSlug::new("test-project").unwrap()),
            label: "Production".to_string(),
        };

        // act
        let result = handler.handle(cmd, &ctx).await;

        // assert
        assert!(result.is_ok());
        assert!(!result.unwrap().as_uuid().is_nil());
    }
}
