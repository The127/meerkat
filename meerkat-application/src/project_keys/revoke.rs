use async_trait::async_trait;

use meerkat_domain::models::permission::ProjectPermission;
use meerkat_domain::models::project::ProjectIdentifier;
use meerkat_domain::models::project_key::ProjectKeyId;

use crate::behaviors::authorization::project_extensions;
use crate::context::RequestContext;
use crate::error::ApplicationError;
use mediator_rs::{Extensions, Request, Handler};

pub struct RevokeProjectKey {
    pub project: ProjectIdentifier,
    pub key_id: ProjectKeyId,
}

impl Request for RevokeProjectKey {
    type Output = ();

    fn extensions(&self) -> Extensions {
        project_extensions(
            "RevokeProjectKey",
            vec![ProjectPermission::ProjectManageKeys.into()],
            self.project.clone(),
        )
    }
}

pub struct RevokeProjectKeyHandler;

#[async_trait]
impl Handler<RevokeProjectKey, ApplicationError, RequestContext> for RevokeProjectKeyHandler {
    async fn handle(
        &self,
        cmd: RevokeProjectKey,
        ctx: &RequestContext,
    ) -> Result<(), ApplicationError> {
        let uow = ctx.uow().await;

        let mut key = uow.project_keys().find(&cmd.key_id).await?;
        key.revoke()?;
        uow.project_keys().save(key);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use meerkat_domain::models::organization::OrganizationId;
    use meerkat_domain::models::project::{ProjectIdentifier, ProjectSlug};
    use meerkat_domain::models::project_key::ProjectKeyStatus;
    use meerkat_domain::testing::test_project_key;

    use crate::context::RequestContext;
    use mediator_rs::Handler;
    use crate::ports::project_key_repository::MockProjectKeyRepository;
    use crate::ports::unit_of_work::MockUnitOfWork;

    use super::{RevokeProjectKey, RevokeProjectKeyHandler};

    #[tokio::test]
    async fn given_active_key_then_finds_revokes_and_saves() {
        // arrange
        let key = test_project_key();
        let key_id = key.id().clone();
        let expected_key_id = key_id.clone();

        let mut key_repo = MockProjectKeyRepository::new();
        key_repo.expect_find()
            .times(1)
            .returning(move |_| Box::pin(std::future::ready(Ok(key.clone()))));
        key_repo.expect_save()
            .times(1)
            .withf(|key| *key.status() == ProjectKeyStatus::Revoked)
            .returning(|_| ());

        let ctx = RequestContext::test()
            .with_scoped_uow(Box::new(
                MockUnitOfWork::new().with_project_key_repo(key_repo),
            ));

        let handler = RevokeProjectKeyHandler;
        let cmd = RevokeProjectKey {
            project: ProjectIdentifier::Slug(OrganizationId::new(), ProjectSlug::new("test-project").unwrap()),
            key_id: expected_key_id,
        };

        // act
        let result = handler.handle(cmd, &ctx).await;

        // assert
        assert!(result.is_ok());
    }
}
