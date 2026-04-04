use async_trait::async_trait;

use meerkat_domain::models::permission::ProjectPermission;
use meerkat_domain::models::project::ProjectIdentifier;
use meerkat_domain::models::project_key::{ProjectKeyId, RateLimit};

use crate::behaviors::authorization::project_extensions;
use crate::context::RequestContext;
use crate::error::ApplicationError;
use crate::extensions::Extensions;
use crate::mediator::{Request, Handler};

pub struct UpdateProjectKeyRateLimit {
    pub project: ProjectIdentifier,
    pub key_id: ProjectKeyId,
    pub rate_limit: Option<u64>,
}

impl Request for UpdateProjectKeyRateLimit {
    type Output = ();

    fn extensions(&self) -> Extensions {
        project_extensions(
            "UpdateProjectKeyRateLimit",
            vec![ProjectPermission::ProjectManageKeys.into()],
            self.project.clone(),
        )
    }
}

pub struct UpdateProjectKeyRateLimitHandler;

#[async_trait]
impl Handler<UpdateProjectKeyRateLimit, ApplicationError, RequestContext> for UpdateProjectKeyRateLimitHandler {
    async fn handle(
        &self,
        cmd: UpdateProjectKeyRateLimit,
        ctx: &RequestContext,
    ) -> Result<(), ApplicationError> {
        let uow = ctx.uow().await;

        let mut key = uow.project_keys().find(&cmd.key_id).await?;

        let limit = cmd.rate_limit
            .map(RateLimit::new)
            .transpose()?;

        key.set_rate_limit(limit)?;
        uow.project_keys().save(key);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use meerkat_domain::models::organization::OrganizationId;
    use meerkat_domain::models::project::{ProjectIdentifier, ProjectSlug};
    use meerkat_domain::testing::test_project_key;

    use crate::context::RequestContext;
    use crate::mediator::Handler;
    use crate::ports::project_key_repository::MockProjectKeyRepository;
    use crate::ports::unit_of_work::MockUnitOfWork;

    use super::{UpdateProjectKeyRateLimit, UpdateProjectKeyRateLimitHandler};

    #[tokio::test]
    async fn given_valid_rate_limit_then_finds_updates_and_saves() {
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
            .withf(|key| key.rate_limit().map(|r| r.value()) == Some(500))
            .returning(|_| ());

        let ctx = RequestContext::test()
            .with_scoped_uow(Box::new(
                MockUnitOfWork::new().with_project_key_repo(key_repo),
            ));

        let handler = UpdateProjectKeyRateLimitHandler;
        let cmd = UpdateProjectKeyRateLimit {
            project: ProjectIdentifier::Slug(OrganizationId::new(), ProjectSlug::new("test-project").unwrap()),
            key_id: expected_key_id,
            rate_limit: Some(500),
        };

        // act
        let result = handler.handle(cmd, &ctx).await;

        // assert
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn given_null_rate_limit_then_clears_rate_limit() {
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
            .withf(|key| key.rate_limit().is_none())
            .returning(|_| ());

        let ctx = RequestContext::test()
            .with_scoped_uow(Box::new(
                MockUnitOfWork::new().with_project_key_repo(key_repo),
            ));

        let handler = UpdateProjectKeyRateLimitHandler;
        let cmd = UpdateProjectKeyRateLimit {
            project: ProjectIdentifier::Slug(OrganizationId::new(), ProjectSlug::new("test-project").unwrap()),
            key_id: expected_key_id,
            rate_limit: None,
        };

        // act
        let result = handler.handle(cmd, &ctx).await;

        // assert
        assert!(result.is_ok());
    }
}
