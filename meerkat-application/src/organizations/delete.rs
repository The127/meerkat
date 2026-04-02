use async_trait::async_trait;

use meerkat_domain::models::organization::OrganizationIdentifier;

use meerkat_domain::models::permission::OrgPermission;

use crate::behaviors::authorization::{CommandName, RequiredPermissions};
use crate::context::RequestContext;
use crate::error::ApplicationError;
use crate::extensions::Extensions;
use crate::mediator::{Command, Handler};

pub struct DeleteOrganization {
    pub identifier: OrganizationIdentifier,
}

impl Command for DeleteOrganization {
    type Output = ();

    fn extensions(&self) -> Extensions {
        let mut ext = Extensions::new();
        ext.insert(CommandName("DeleteOrganization".to_string()));
        ext.insert(RequiredPermissions(vec![OrgPermission::OrgDelete.into()]));
        ext
    }
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

        let org = uow.organizations().find(&cmd.identifier).await?;

        uow.organizations().delete(org.id().clone());

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use meerkat_domain::models::organization::OrganizationIdentifier;
    use meerkat_domain::testing::test_org;

    use crate::context::RequestContext;
    use crate::mediator::Handler;
    use crate::ports::organization_repository::MockOrganizationRepository;
    use crate::ports::unit_of_work::MockUnitOfWork;

    use super::{DeleteOrganization, DeleteOrganizationHandler};

    #[tokio::test]
    async fn given_existing_org_then_finds_and_deletes() {
        // arrange
        let (org, _clock) = test_org();
        let org_id = org.id().clone();
        let expected_id = org_id.clone();
        let expected_delete_id = org_id.clone();

        let mut repo = MockOrganizationRepository::new();
        repo.expect_find()
            .times(1)
            .withf(move |identifier| matches!(identifier, OrganizationIdentifier::Id(id) if *id == expected_id))
            .returning(move |_| Box::pin(std::future::ready(Ok(org.clone()))));
        repo.expect_delete()
            .times(1)
            .withf(move |id| *id == expected_delete_id)
            .returning(|_| ());

        let ctx = RequestContext::test()
            .with_scoped_uow(Box::new(MockUnitOfWork::new().with_organization_repo(repo)));

        let handler = DeleteOrganizationHandler;
        let cmd = DeleteOrganization {
            identifier: OrganizationIdentifier::Id(org_id),
        };

        // act
        let result = handler.handle(cmd, &ctx).await;

        // assert
        assert!(result.is_ok());
    }
}
