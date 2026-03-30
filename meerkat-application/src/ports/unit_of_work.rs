use async_trait::async_trait;

use crate::error::ApplicationError;
use crate::ports::organization_store::WriteOrganizationStore;

#[async_trait]
pub trait UnitOfWork: Send {
    fn organizations(&self) -> &dyn WriteOrganizationStore;
    async fn save_changes(&mut self) -> Result<(), ApplicationError>;
}

#[cfg_attr(any(test, feature = "test-utils"), mockall::automock)]
#[async_trait]
pub trait UnitOfWorkFactory: Send + Sync {
    async fn create(&self) -> Result<Box<dyn UnitOfWork>, ApplicationError>;
}

// UnitOfWork can't use #[automock] because organizations() returns &dyn trait.
// Hand-rolled mock embeds MockWriteOrganizationStore from mockall.
#[cfg(any(test, feature = "test-utils"))]
pub struct MockUnitOfWork {
    org_store: crate::ports::organization_store::MockWriteOrganizationStore,
    save_changes_result: Option<Result<(), ApplicationError>>,
}

#[cfg(any(test, feature = "test-utils"))]
impl Default for MockUnitOfWork {
    fn default() -> Self {
        Self {
            org_store: crate::ports::organization_store::MockWriteOrganizationStore::new(),
            save_changes_result: Some(Ok(())),
        }
    }
}

#[cfg(any(test, feature = "test-utils"))]
impl MockUnitOfWork {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_organization_store(mut self, store: crate::ports::organization_store::MockWriteOrganizationStore) -> Self {
        self.org_store = store;
        self
    }

    pub fn with_save_changes_result(mut self, result: Result<(), ApplicationError>) -> Self {
        self.save_changes_result = Some(result);
        self
    }
}

#[cfg(any(test, feature = "test-utils"))]
#[async_trait]
impl UnitOfWork for MockUnitOfWork {
    fn organizations(&self) -> &dyn WriteOrganizationStore {
        &self.org_store
    }

    async fn save_changes(&mut self) -> Result<(), ApplicationError> {
        self.save_changes_result
            .take()
            .unwrap_or(Ok(()))
    }
}
