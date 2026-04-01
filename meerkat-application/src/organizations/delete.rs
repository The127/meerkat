use async_trait::async_trait;

use meerkat_domain::models::organization::OrganizationId;

use crate::context::RequestContext;
use crate::error::ApplicationError;
use crate::mediator::{Command, Handler};

pub struct DeleteOrganization {
    pub organization_id: OrganizationId,
}

impl Command for DeleteOrganization {
    type Output = ();
}

pub struct DeleteOrganizationHandler;

#[async_trait]
impl Handler<DeleteOrganization, ApplicationError, RequestContext> for DeleteOrganizationHandler {
    async fn handle(
        &self,
        cmd: DeleteOrganization,
        ctx: &RequestContext,
    ) -> Result<(), ApplicationError> {
        let uow = ctx.uow().await;

        let _ = uow.organizations().find_by_id(&cmd.organization_id).await?;

        uow.organizations().delete(cmd.organization_id);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use meerkat_domain::testing::test_org;

    use crate::context::RequestContext;
    use crate::mediator::Handler;
    use crate::ports::organization_repository::MockOrganizationRepository;
    use crate::ports::unit_of_work::MockUnitOfWork;

    use super::{DeleteOrganization, DeleteOrganizationHandler};

    #[tokio::test]
    async fn given_existing_org_then_fetches_by_id_and_deletes() {
        // arrange
        let (org, _clock) = test_org();
        let org_id = org.id().clone();
        let expected_find_id = org_id.clone();
        let expected_delete_id = org_id.clone();

        let mut repo = MockOrganizationRepository::new();
        repo.expect_find_by_id()
            .times(1)
            .withf(move |id| *id == expected_find_id)
            .returning(move |_| Box::pin(std::future::ready(Ok(org.clone()))));
        repo.expect_delete()
            .times(1)
            .withf(move |id| *id == expected_delete_id)
            .returning(|_| ());

        let ctx = RequestContext::test()
            .with_scoped_uow(Box::new(MockUnitOfWork::new().with_organization_repo(repo)));

        let handler = DeleteOrganizationHandler;
        let cmd = DeleteOrganization {
            organization_id: org_id,
        };

        // act
        let result = handler.handle(cmd, &ctx).await;

        // assert
        assert!(result.is_ok());
    }
}
