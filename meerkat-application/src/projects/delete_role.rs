use async_trait::async_trait;

use meerkat_domain::models::permission::ProjectPermission;
use meerkat_domain::models::project::ProjectIdentifier;
use meerkat_domain::models::project_role::ProjectRoleId;

use crate::behaviors::authorization::project_extensions;
use crate::context::RequestContext;
use crate::error::ApplicationError;
use crate::extensions::Extensions;
use crate::mediator::{Handler, Request};

pub struct DeleteProjectRole {
    pub project: ProjectIdentifier,
    pub role_id: ProjectRoleId,
}

impl Request for DeleteProjectRole {
    type Output = ();

    fn extensions(&self) -> Extensions {
        project_extensions(
            "DeleteProjectRole",
            vec![ProjectPermission::ProjectManageMembers.into()],
            self.project.clone(),
        )
    }
}

pub struct DeleteProjectRoleHandler;

#[async_trait]
impl Handler<DeleteProjectRole, ApplicationError, RequestContext> for DeleteProjectRoleHandler {
    async fn handle(
        &self,
        cmd: DeleteProjectRole,
        ctx: &RequestContext,
    ) -> Result<(), ApplicationError> {
        let uow = ctx.uow().await;

        uow.project_roles().find(&cmd.role_id).await?;
        uow.project_roles().delete(cmd.role_id);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use vec1::vec1;

    use meerkat_domain::models::permission::ProjectPermission;
    use meerkat_domain::models::project::ProjectIdentifier;
    use meerkat_domain::models::project_role::{ProjectRole, ProjectRoleId, ProjectRoleSlug};
    use meerkat_domain::models::project::ProjectId;

    use crate::context::RequestContext;
    use crate::mediator::Handler;
    use crate::ports::project_role_repository::MockProjectRoleRepository;
    use crate::ports::unit_of_work::MockUnitOfWork;

    use super::{DeleteProjectRole, DeleteProjectRoleHandler};

    #[tokio::test]
    async fn given_existing_role_then_finds_and_deletes() {
        // arrange
        let role = ProjectRole::new(
            ProjectId::new(),
            "Temporary".into(),
            ProjectRoleSlug::new("temporary").unwrap(),
            vec1![ProjectPermission::ProjectRead],
            false,
        ).unwrap();
        let role_id = role.id().clone();
        let expected_id = role_id.clone();
        let expected_delete_id = role_id.clone();

        let mut role_repo = MockProjectRoleRepository::new();
        role_repo
            .expect_find()
            .times(1)
            .withf(move |id| *id == expected_id)
            .returning(move |_| Box::pin(std::future::ready(Ok(role.clone()))));
        role_repo
            .expect_delete()
            .times(1)
            .withf(move |id| *id == expected_delete_id)
            .returning(|_| ());

        let ctx = RequestContext::test().with_scoped_uow(Box::new(
            MockUnitOfWork::new().with_project_role_repo(role_repo),
        ));

        let cmd = DeleteProjectRole {
            project: ProjectIdentifier::Id(ProjectId::new()),
            role_id,
        };

        // act
        let result = DeleteProjectRoleHandler.handle(cmd, &ctx).await;

        // assert
        assert!(result.is_ok());
    }
}
