use async_trait::async_trait;

use meerkat_domain::models::member::MemberId;
use meerkat_domain::models::permission::ProjectPermission;
use meerkat_domain::models::project::ProjectIdentifier;
use meerkat_domain::models::project_role::ProjectRoleId;

use crate::behaviors::authorization::project_extensions;
use crate::context::RequestContext;
use crate::error::ApplicationError;
use crate::extensions::Extensions;
use crate::mediator::{Handler, Request};

pub struct RemoveRoleFromProjectMember {
    pub project: ProjectIdentifier,
    pub member_id: MemberId,
    pub role_id: ProjectRoleId,
}

impl Request for RemoveRoleFromProjectMember {
    type Output = ();

    fn extensions(&self) -> Extensions {
        project_extensions(
            "RemoveRoleFromProjectMember",
            vec![ProjectPermission::ProjectManageMembers.into()],
            self.project.clone(),
        )
    }
}

pub struct RemoveRoleFromProjectMemberHandler;

#[async_trait]
impl Handler<RemoveRoleFromProjectMember, ApplicationError, RequestContext>
    for RemoveRoleFromProjectMemberHandler
{
    async fn handle(
        &self,
        cmd: RemoveRoleFromProjectMember,
        ctx: &RequestContext,
    ) -> Result<(), ApplicationError> {
        let uow = ctx.uow().await;

        let project = uow.projects().find(&cmd.project).await?;

        let mut member = uow
            .project_members()
            .find_by_project_and_member(project.id(), &cmd.member_id)
            .await?
            .ok_or(ApplicationError::NotFound)?;

        member.remove_role(&cmd.role_id);
        uow.project_members().save(member);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use meerkat_domain::models::member::MemberId;
    use meerkat_domain::models::project::ProjectIdentifier;
    use meerkat_domain::models::project_member::ProjectMember;
    use meerkat_domain::models::project_role::ProjectRoleId;
    use meerkat_domain::testing::test_project;

    use crate::context::RequestContext;
    use crate::mediator::Handler;
    use crate::ports::project_member_repository::MockProjectMemberRepository;
    use crate::ports::project_repository::MockProjectRepository;
    use crate::ports::unit_of_work::MockUnitOfWork;

    use super::{RemoveRoleFromProjectMember, RemoveRoleFromProjectMemberHandler};

    #[tokio::test]
    async fn given_member_with_role_then_removes_role_and_saves() {
        // arrange
        let project = test_project();
        let project_id = project.id().clone();
        let member_id = MemberId::new();
        let role_id = ProjectRoleId::new();

        let mut member = ProjectMember::new(member_id.clone(), project_id.clone(), vec![]);
        member.assign_role(role_id.clone());

        let removed_role_id = role_id.clone();

        let mut project_repo = MockProjectRepository::new();
        project_repo
            .expect_find()
            .times(1)
            .returning(move |_| Box::pin(std::future::ready(Ok(project.clone()))));

        let mut member_repo = MockProjectMemberRepository::new();
        member_repo
            .expect_find_by_project_and_member()
            .times(1)
            .returning(move |_, _| Box::pin(std::future::ready(Ok(Some(member.clone())))));
        member_repo
            .expect_save()
            .times(1)
            .withf(move |m| !m.role_ids().contains(&removed_role_id))
            .returning(|_| ());

        let ctx = RequestContext::test().with_scoped_uow(Box::new(
            MockUnitOfWork::new()
                .with_project_repo(project_repo)
                .with_project_member_repo(member_repo),
        ));

        let cmd = RemoveRoleFromProjectMember {
            project: ProjectIdentifier::Id(project_id),
            member_id,
            role_id,
        };

        // act
        let result = RemoveRoleFromProjectMemberHandler.handle(cmd, &ctx).await;

        // assert
        assert!(result.is_ok());
    }
}
