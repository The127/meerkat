use async_trait::async_trait;

use meerkat_domain::models::project_key::ProjectKey;

use crate::context::RequestContext;
use crate::error::ApplicationError;
use crate::events::{DomainEvent, DomainEventHandler};

pub struct GenerateProjectKeyOnProjectCreated;

#[async_trait]
impl DomainEventHandler for GenerateProjectKeyOnProjectCreated {
    async fn handle(&self, event: &DomainEvent, ctx: &RequestContext) -> Result<(), ApplicationError> {
        let DomainEvent::ProjectCreated { project_id } = event else {
            return Ok(());
        };

        let key = ProjectKey::generate(project_id.clone(), "Default".into())?;
        ctx.uow().await.project_keys().add(key);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use meerkat_domain::models::project::ProjectId;

    use crate::context::RequestContext;
    use crate::events::DomainEvent;
    use crate::events::DomainEventHandler;
    use crate::ports::project_key_repository::MockProjectKeyRepository;
    use crate::ports::unit_of_work::MockUnitOfWork;

    use super::GenerateProjectKeyOnProjectCreated;

    #[tokio::test]
    async fn given_project_created_event_then_generates_default_key() {
        // arrange
        let project_id = ProjectId::new();
        let expected_project_id = project_id.clone();

        let mut key_repo = MockProjectKeyRepository::new();
        key_repo.expect_add()
            .times(1)
            .withf(move |key| {
                *key.project_id() == expected_project_id
                    && key.label() == "Default"
                    && key.key_token().as_str().len() == 32
            })
            .returning(|_| ());

        let ctx = RequestContext::test()
            .with_scoped_uow(Box::new(MockUnitOfWork::new().with_project_key_repo(key_repo)));

        let handler = GenerateProjectKeyOnProjectCreated;
        let event = DomainEvent::ProjectCreated { project_id };

        // act
        let result = handler.handle(&event, &ctx).await;

        // assert
        assert!(result.is_ok());
    }
}
