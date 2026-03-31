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
    pub jwks_url: Option<Url>,
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
            oidc.name, oidc.client_id, oidc.issuer_url, oidc.audience, oidc.jwks_url,
            ctx.clock(),
        ).map_err(|e| ApplicationError::Validation(e.to_string()))?;

        let org = Organization::new(
            cmd.name, cmd.slug,
            oidc_config,
            ctx.clock(),
        ).map_err(|e| ApplicationError::Validation(e.to_string()))?;

        let id = org.id().clone();

        ctx.with_uow(|uow| uow.organizations().add(org));

        Ok(id)
    }
}

#[cfg(test)]
mod tests {
    use meerkat_domain::models::oidc_config::{Audience, ClientId, Url};
    use meerkat_domain::models::organization::OrganizationSlug;

    use crate::context::RequestContext;
    use crate::error::ApplicationError;
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
                jwks_url: None,
            },
        }
    }

    #[tokio::test]
    async fn given_valid_input_when_creating_organization_it_should_return_an_id() {
        // arrange
        let mut repo = MockOrganizationRepository::new();
        repo.expect_add().times(1).returning(|_| ());

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

    #[tokio::test]
    async fn given_empty_name_when_creating_organization_it_should_return_validation_error() {
        // arrange
        let ctx = RequestContext::test();
        let handler = CreateOrganizationHandler;
        let mut cmd = valid_cmd();
        cmd.name = "  ".to_string();

        // act
        let result = handler.handle(cmd, &ctx).await;

        // assert
        match result {
            Err(ApplicationError::Validation(_)) => (),
            other => panic!("Expected Validation error, got {:?}", other),
        }
    }
}
