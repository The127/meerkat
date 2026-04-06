use async_trait::async_trait;
use vec1::Vec1;

use meerkat_domain::models::permission::ProjectPermission;
use meerkat_domain::models::project::ProjectIdentifier;
use meerkat_domain::models::project_role::{ProjectRole, ProjectRoleSlug};

use crate::behaviors::authorization::project_extensions;
use crate::context::RequestContext;
use crate::error::ApplicationError;
use crate::extensions::Extensions;
use crate::mediator::{Handler, Request};

pub struct CreateProjectRole {
    pub project: ProjectIdentifier,
    pub name: String,
    pub permissions: Vec1<ProjectPermission>,
}

impl Request for CreateProjectRole {
    type Output = ProjectRole;

    fn extensions(&self) -> Extensions {
        project_extensions(
            "CreateProjectRole",
            vec![ProjectPermission::ProjectManageMembers.into()],
            self.project.clone(),
        )
    }
}

pub struct CreateProjectRoleHandler;

fn slugify(name: &str) -> String {
    let slug: String = name
        .to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect();
    let slug = slug.trim_matches('-').to_string();
    let mut result = String::with_capacity(slug.len());
    let mut prev_dash = false;
    for c in slug.chars() {
        if c == '-' {
            if !prev_dash {
                result.push(c);
            }
            prev_dash = true;
        } else {
            result.push(c);
            prev_dash = false;
        }
    }
    result
}

#[async_trait]
impl Handler<CreateProjectRole, ApplicationError, RequestContext> for CreateProjectRoleHandler {
    async fn handle(
        &self,
        cmd: CreateProjectRole,
        ctx: &RequestContext,
    ) -> Result<ProjectRole, ApplicationError> {
        let uow = ctx.uow().await;

        let project = uow.projects().find(&cmd.project).await?;

        let slug_str = slugify(&cmd.name);
        let slug = ProjectRoleSlug::new(&slug_str)
            .map_err(|e| ApplicationError::Validation(e.to_string()))?;

        let role = ProjectRole::new(project.id().clone(), cmd.name, slug, cmd.permissions, false)
            .map_err(|e| ApplicationError::Validation(e.to_string()))?;

        uow.project_roles().add(role.clone());

        Ok(role)
    }
}

#[cfg(test)]
mod tests {
    use vec1::vec1;

    use meerkat_domain::models::permission::ProjectPermission;
    use meerkat_domain::models::project::ProjectIdentifier;
    use meerkat_domain::testing::test_project;

    use crate::context::RequestContext;
    use crate::mediator::Handler;
    use crate::ports::project_repository::MockProjectRepository;
    use crate::ports::project_role_repository::MockProjectRoleRepository;
    use crate::ports::unit_of_work::MockUnitOfWork;

    use super::{CreateProjectRole, CreateProjectRoleHandler};

    #[tokio::test]
    async fn given_valid_input_then_role_is_created_and_added() {
        // arrange
        let project = test_project();
        let project_id = project.id().clone();

        let mut project_repo = MockProjectRepository::new();
        project_repo
            .expect_find()
            .times(1)
            .returning(move |_| Box::pin(std::future::ready(Ok(project.clone()))));

        let mut role_repo = MockProjectRoleRepository::new();
        role_repo
            .expect_add()
            .times(1)
            .withf(|role| role.name() == "My Role" && !role.is_default())
            .returning(|_| ());

        let ctx = RequestContext::test().with_scoped_uow(Box::new(
            MockUnitOfWork::new()
                .with_project_repo(project_repo)
                .with_project_role_repo(role_repo),
        ));

        let cmd = CreateProjectRole {
            project: ProjectIdentifier::Id(project_id),
            name: "My Role".into(),
            permissions: vec1![ProjectPermission::ProjectRead],
        };

        // act
        let result = CreateProjectRoleHandler.handle(cmd, &ctx).await;

        // assert
        assert!(result.is_ok());
        let role = result.unwrap();
        assert_eq!(role.name(), "My Role");
        assert_eq!(role.slug().as_str(), "my-role");
    }
}
