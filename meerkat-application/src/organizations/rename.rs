use async_trait::async_trait;

use meerkat_domain::models::organization::OrganizationIdentifier;

use crate::context::RequestContext;
use crate::error::ApplicationError;
use crate::mediator::{Command, Handler};

pub struct RenameOrganization {
    pub identifier: OrganizationIdentifier,
    pub name: String,
}

impl Command for RenameOrganization {
    type Output = ();
}

pub struct RenameOrganizationHandler;

#[async_trait]
impl Handler<RenameOrganization, ApplicationError, RequestContext> for RenameOrganizationHandler {
    async fn handle(
        &self,
        cmd: RenameOrganization,
        ctx: &RequestContext,
    ) -> Result<(), ApplicationError> {
        let uow = ctx.uow().await;

        let mut org = uow.organizations().find(&cmd.identifier).await?;

        org.update_name(cmd.name)?;

        uow.organizations().save(org);

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

    use super::{RenameOrganization, RenameOrganizationHandler};

    #[tokio::test]
    async fn given_valid_name_then_finds_org_and_saves_renamed() {
        // arrange
        let (org, _clock) = test_org();
        let org_id = org.id().clone();
        let expected_id = org_id.clone();

        let mut repo = MockOrganizationRepository::new();
        repo.expect_find()
            .times(1)
            .withf(move |identifier| matches!(identifier, OrganizationIdentifier::Id(id) if *id == expected_id))
            .returning(move |_| Box::pin(std::future::ready(Ok(org.clone()))));
        repo.expect_save()
            .times(1)
            .withf(|org| org.name() == "New Name")
            .returning(|_| ());

        let ctx = RequestContext::test()
            .with_scoped_uow(Box::new(MockUnitOfWork::new().with_organization_repo(repo)));

        let handler = RenameOrganizationHandler;
        let cmd = RenameOrganization {
            identifier: OrganizationIdentifier::Id(org_id),
            name: "New Name".to_string(),
        };

        // act
        let result = handler.handle(cmd, &ctx).await;

        // assert
        assert!(result.is_ok());
    }
}
