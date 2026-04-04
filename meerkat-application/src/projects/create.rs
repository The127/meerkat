use async_trait::async_trait;

use meerkat_domain::models::organization::OrganizationId;
use meerkat_domain::models::project::{Project, ProjectId, ProjectSlug};
use meerkat_domain::models::project_member::ProjectMember;
use meerkat_domain::models::project_role::ProjectRole;

use meerkat_domain::models::permission::OrgPermission;

use crate::behaviors::authorization::org_extensions;
use crate::context::RequestContext;
use crate::error::ApplicationError;
use crate::events::DomainEvent;
use crate::extensions::Extensions;
use crate::mediator::{Request, Handler};

pub struct CreateProject {
    pub organization_id: OrganizationId,
    pub name: String,
    pub slug: ProjectSlug,
}

impl Request for CreateProject {
    type Output = ProjectId;

    fn extensions(&self) -> Extensions {
        org_extensions("CreateProject", vec![OrgPermission::OrgCreateProject.into()])
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
        let project = Project::new(cmd.organization_id, cmd.name, cmd.slug)?;
        let project_id = project.id().clone();

        let (default_roles, admin_role_id) = ProjectRole::default_roles(project_id.clone());

        let uow = ctx.uow().await;
        uow.projects().add(project);

        for role in default_roles {
            uow.project_roles().add(role);
        }

        if let Some(auth) = ctx.auth() {
            let member = ProjectMember::new(
                auth.member_id.clone(),
                project_id.clone(),
                vec![admin_role_id],
            );
            uow.project_members().add(member);
        }

        ctx.raise(DomainEvent::ProjectCreated { project_id: project_id.clone() }).await;

        Ok(project_id)
    }
}

#[cfg(test)]
mod tests {
    use meerkat_domain::models::organization::OrganizationId;
    use meerkat_domain::models::project::ProjectSlug;

    use crate::context::RequestContext;
    use crate::events::DomainEvent;
    use crate::mediator::Handler;
    use crate::ports::project_repository::MockProjectRepository;
    use crate::ports::unit_of_work::MockUnitOfWork;

    use super::{CreateProject, CreateProjectHandler};

    #[tokio::test]
    async fn given_valid_input_then_adds_project_and_raises_project_created_event() {
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
        let project_id = handler.handle(cmd, &ctx).await.expect("handler should succeed");

        // assert
        assert!(!project_id.as_uuid().is_nil());

        let events = ctx.drain_events().await;
        assert_eq!(events.len(), 1);
        match &events[0] {
            DomainEvent::ProjectCreated { project_id: event_pid } => {
                assert_eq!(event_pid, &project_id);
            }
        }
    }
}
