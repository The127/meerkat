use async_trait::async_trait;
use vec1::Vec1;

use meerkat_domain::models::permission::ProjectPermission;
use meerkat_domain::models::project::ProjectIdentifier;
use meerkat_domain::models::project_role::ProjectRoleId;

use crate::behaviors::authorization::project_extensions;
use crate::context::RequestContext;
use crate::error::ApplicationError;
use crate::extensions::Extensions;
use crate::mediator::{Handler, Request};

pub struct UpdateProjectRole {
    pub project: ProjectIdentifier,
    pub role_id: ProjectRoleId,
    pub name: String,
    pub permissions: Vec1<ProjectPermission>,
}

impl Request for UpdateProjectRole {
    type Output = ();

    fn extensions(&self) -> Extensions {
        project_extensions(
            "UpdateProjectRole",
            vec![ProjectPermission::ProjectManageMembers.into()],
            self.project.clone(),
        )
    }
}

pub struct UpdateProjectRoleHandler;

#[async_trait]
impl Handler<UpdateProjectRole, ApplicationError, RequestContext> for UpdateProjectRoleHandler {
    async fn handle(
        &self,
        cmd: UpdateProjectRole,
        ctx: &RequestContext,
    ) -> Result<(), ApplicationError> {
        let uow = ctx.uow().await;

        let mut role = uow.project_roles().find(&cmd.role_id).await?;
        role.update(cmd.name, cmd.permissions)
            .map_err(|e| ApplicationError::Validation(e.to_string()))?;
        uow.project_roles().save(role);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use vec1::vec1;

    use meerkat_domain::models::permission::ProjectPermission;
    use meerkat_domain::models::project::ProjectIdentifier;
    use meerkat_domain::models::project_role::{ProjectRole, ProjectRoleSlug};
    use meerkat_domain::models::project::ProjectId;

    use crate::context::RequestContext;
    use crate::mediator::Handler;
    use crate::ports::project_role_repository::MockProjectRoleRepository;
    use crate::ports::unit_of_work::MockUnitOfWork;

    use super::{UpdateProjectRole, UpdateProjectRoleHandler};

    fn test_role() -> ProjectRole {
        ProjectRole::new(
            ProjectId::new(),
            "Original".into(),
            ProjectRoleSlug::new("original").unwrap(),
            vec1![ProjectPermission::ProjectRead],
            false,
        ).unwrap()
    }

    #[tokio::test]
    async fn given_existing_role_then_finds_updates_and_saves() {
        // arrange
        let role = test_role();
        let role_id = role.id().clone();
        let expected_role_id = role_id.clone();

        let mut role_repo = MockProjectRoleRepository::new();
        role_repo
            .expect_find()
            .times(1)
            .withf(move |id| *id == expected_role_id)
            .returning(move |_| Box::pin(std::future::ready(Ok(role.clone()))));
        role_repo
            .expect_save()
            .times(1)
            .withf(|r| r.name() == "Renamed")
            .returning(|_| ());

        let ctx = RequestContext::test().with_scoped_uow(Box::new(
            MockUnitOfWork::new().with_project_role_repo(role_repo),
        ));

        let cmd = UpdateProjectRole {
            project: ProjectIdentifier::Id(ProjectId::new()),
            role_id,
            name: "Renamed".into(),
            permissions: vec1![ProjectPermission::ProjectRead, ProjectPermission::ProjectWrite],
        };

        // act
        let result = UpdateProjectRoleHandler.handle(cmd, &ctx).await;

        // assert
        assert!(result.is_ok());
    }
}
