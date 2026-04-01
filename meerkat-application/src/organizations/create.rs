use async_trait::async_trait;

use meerkat_domain::models::oidc_config::{Audience, ClientId, OidcConfig, Url};
use meerkat_domain::models::organization::{Organization, OrganizationId, OrganizationSlug};

use crate::context::RequestContext;
use crate::error::ApplicationError;
use crate::mediator::{Command, Handler};

pub struct CreateOrganizationOidcConfig {
    pub name: String,
    pub client_id: ClientId,
    pub issuer_url: Url,
    pub audience: Audience,
    pub discovery_url: Option<Url>,
}

pub struct CreateOrganization {
    pub name: String,
    pub slug: OrganizationSlug,
    pub oidc_config: CreateOrganizationOidcConfig,
}

impl Command for CreateOrganization {
    type Output = OrganizationId;
}

pub struct CreateOrganizationHandler;

#[async_trait]
impl Handler<CreateOrganization, ApplicationError, RequestContext> for CreateOrganizationHandler {
    async fn handle(
        &self,
        cmd: CreateOrganization,
        ctx: &RequestContext,
    ) -> Result<OrganizationId, ApplicationError> {
        let oidc = cmd.oidc_config;
        let oidc_config = OidcConfig::new(
            oidc.name, oidc.client_id, oidc.issuer_url, oidc.audience, oidc.discovery_url,
            ctx.clock(),
        )?;

        let org = Organization::new(
            cmd.name, cmd.slug,
            oidc_config,
            ctx.clock(),
        )?;

        let id = org.id().clone();

        ctx.uow().await.organizations().add(org);

        Ok(id)
    }
}

#[cfg(test)]
mod tests {
    use meerkat_domain::models::oidc_config::{Audience, ClientId, Url};
    use meerkat_domain::models::organization::OrganizationSlug;

    use crate::context::RequestContext;
    use crate::mediator::Handler;
    use crate::ports::organization_repository::MockOrganizationRepository;
    use crate::ports::unit_of_work::MockUnitOfWork;

    use super::{CreateOrganization, CreateOrganizationHandler, CreateOrganizationOidcConfig};

    fn valid_cmd() -> CreateOrganization {
        CreateOrganization {
            name: "Meerkat Inc.".to_string(),
            slug: OrganizationSlug::new("meerkat-inc").unwrap(),
            oidc_config: CreateOrganizationOidcConfig {
                name: "Default SSO".to_string(),
                client_id: ClientId::new("meerkat-client").unwrap(),
                issuer_url: Url::new("https://auth.example.com").unwrap(),
                audience: Audience::new("meerkat-api").unwrap(),
                discovery_url: None,
            },
        }
    }

    #[tokio::test]
    async fn given_valid_input_then_adds_org_with_correct_fields_and_returns_id() {
        // arrange
        let mut repo = MockOrganizationRepository::new();
        repo.expect_add()
            .times(1)
            .withf(|org| {
                org.name() == "Meerkat Inc."
                    && org.slug().as_str() == "meerkat-inc"
                    && org.oidc_configs().len() == 1
                    && org.oidc_configs()[0].name() == "Default SSO"
                    && org.oidc_configs()[0].client_id() == &ClientId::new("meerkat-client").unwrap()
                    && org.oidc_configs()[0].issuer_url() == &Url::new("https://auth.example.com").unwrap()
                    && org.oidc_configs()[0].audience() == &Audience::new("meerkat-api").unwrap()
            })
            .returning(|_| ());

        let ctx = RequestContext::test()
            .with_scoped_uow(Box::new(MockUnitOfWork::new().with_organization_repo(repo)));

        let handler = CreateOrganizationHandler;
        let cmd = valid_cmd();

        // act
        let result = handler.handle(cmd, &ctx).await;

        // assert
        assert!(result.is_ok());
        assert!(!result.unwrap().as_uuid().is_nil());
    }
}
