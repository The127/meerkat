use async_trait::async_trait;

use meerkat_domain::models::organization::{Organization, OrganizationId, OrganizationSlug};

use crate::context::AppContext;
use crate::error::ApplicationError;
use crate::mediator::{Command, Handler};

pub struct CreateOrganization {
    pub name: String,
    pub slug: OrganizationSlug,
}

impl Command for CreateOrganization {
    type Output = OrganizationId;
}

pub struct CreateOrganizationHandler;

#[async_trait]
impl Handler<CreateOrganization, ApplicationError, AppContext> for CreateOrganizationHandler {
    async fn handle(
        &self,
        cmd: CreateOrganization,
        ctx: &AppContext,
    ) -> Result<OrganizationId, ApplicationError> {
        let org = Organization::new(cmd.name, cmd.slug, ctx.clock.as_ref())
            .map_err(|e| ApplicationError::Validation(e.to_string()))?;

        let id = org.id().clone();

        let mut uow = ctx.uow_factory.create().await?;
        uow.organizations().insert(org);
        uow.save_changes().await?;

        Ok(id)
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;
    use std::sync::Arc;

    use meerkat_domain::models::organization::OrganizationSlug;
    use meerkat_domain::ports::clock::MockClock;

    use crate::context::AppContext;
    use crate::error::ApplicationError;
    use crate::mediator::Handler;
    use crate::ports::error_observer::ErrorPipeline;
    use crate::ports::organization_store::MockWriteOrganizationStore;
    use crate::ports::unit_of_work::{MockUnitOfWork, MockUnitOfWorkFactory};

    use super::{CreateOrganization, CreateOrganizationHandler};

    fn test_context() -> AppContext {
        let mut factory = MockUnitOfWorkFactory::new();
        factory.expect_create().returning(|| {
            let mut store = MockWriteOrganizationStore::new();
            store.expect_insert().times(1).returning(|_| ());

            Ok(Box::new(MockUnitOfWork::new_with_store(store)))
        });

        AppContext {
            clock: Arc::new(MockClock::new(chrono::Utc::now())),
            uow_factory: Arc::new(factory),
            error_observer: Arc::new(ErrorPipeline::new(vec![])),
        }
    }

    #[tokio::test]
    async fn given_valid_input_when_creating_organization_it_should_return_an_id() {
        // arrange
        let ctx = test_context();
        let handler = CreateOrganizationHandler;
        let cmd = CreateOrganization {
            name: "Meerkat Inc.".to_string(),
            slug: OrganizationSlug::from_str("meerkat-inc").unwrap(),
        };

        // act
        let result = handler.handle(cmd, &ctx).await;

        // assert
        assert!(result.is_ok());
        assert!(!result.unwrap().as_uuid().is_nil());
    }

    #[tokio::test]
    async fn given_empty_name_when_creating_organization_it_should_return_validation_error() {
        // arrange
        let ctx = test_context();
        let handler = CreateOrganizationHandler;
        let cmd = CreateOrganization {
            name: "  ".to_string(),
            slug: OrganizationSlug::from_str("some-slug").unwrap(),
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
