use async_trait::async_trait;

use meerkat_domain::models::organization::OrganizationId;

use crate::context::RequestContext;
use crate::error::ApplicationError;
use crate::mediator::{Command, Handler};

pub struct RenameOrganization {
    pub organization_id: OrganizationId,
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

        let mut org = uow.organizations().find_by_id(&cmd.organization_id).await?;

        org.update_name(cmd.name)?;

        uow.organizations().save(org);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use meerkat_domain::models::oidc_config::{Audience, ClientId, OidcConfig, Url};
    use meerkat_domain::models::organization::{Organization, OrganizationSlug};
    use meerkat_domain::ports::clock::MockClock;

    use crate::context::RequestContext;
    use crate::mediator::Handler;
    use crate::ports::organization_repository::MockOrganizationRepository;
    use crate::ports::unit_of_work::MockUnitOfWork;

    use super::{RenameOrganization, RenameOrganizationHandler};

    fn test_org() -> (Organization, MockClock) {
        let clock = MockClock::new(chrono::Utc::now());
        let config = OidcConfig::new(
            "Default SSO".into(),
            ClientId::new("meerkat-client").unwrap(),
            Url::new("https://auth.example.com").unwrap(),
            Audience::new("meerkat-api").unwrap(),
            None,
            &clock,
        )
        .unwrap();
        let slug = OrganizationSlug::new("test-org").unwrap();
        let org = Organization::new("Test Org".into(), slug, config, &clock).unwrap();
        (org, clock)
    }

    #[tokio::test]
    async fn given_valid_name_then_fetches_by_id_and_saves_renamed_org() {
        // arrange
        let (org, _clock) = test_org();
        let org_id = org.id().clone();
        let expected_id = org_id.clone();

        let mut repo = MockOrganizationRepository::new();
        repo.expect_find_by_id()
            .times(1)
            .withf(move |id| *id == expected_id)
            .returning(move |_| Box::pin(std::future::ready(Ok(org.clone()))));
        repo.expect_save()
            .times(1)
            .withf(|org| org.name() == "New Name")
            .returning(|_| ());

        let ctx = RequestContext::test()
            .with_scoped_uow(Box::new(MockUnitOfWork::new().with_organization_repo(repo)));

        let handler = RenameOrganizationHandler;
        let cmd = RenameOrganization {
            organization_id: org_id,
            name: "New Name".to_string(),
        };

        // act
        let result = handler.handle(cmd, &ctx).await;

        // assert
        assert!(result.is_ok());
    }
}
