use async_trait::async_trait;

use crate::error::ApplicationError;
use crate::ports::organization_store::WriteOrganizationStore;
use crate::ports::project_store::WriteProjectStore;

#[async_trait]
pub trait UnitOfWork: Send {
    fn organizations(&self) -> &dyn WriteOrganizationStore;
    fn projects(&self) -> &dyn WriteProjectStore;
    async fn save_changes(&mut self) -> Result<(), ApplicationError>;
}

#[cfg_attr(any(test, feature = "test-utils"), mockall::automock)]
#[async_trait]
pub trait UnitOfWorkFactory: Send + Sync {
    async fn create(&self) -> Result<Box<dyn UnitOfWork>, ApplicationError>;
}

// UnitOfWork can't use #[automock] because it returns &dyn trait references.
#[cfg(any(test, feature = "test-utils"))]
pub struct MockUnitOfWork {
    org_store: crate::ports::organization_store::MockWriteOrganizationStore,
    project_store: crate::ports::project_store::MockWriteProjectStore,
    save_changes_result: Option<Result<(), ApplicationError>>,
}

#[cfg(any(test, feature = "test-utils"))]
impl Default for MockUnitOfWork {
    fn default() -> Self {
        Self {
            org_store: crate::ports::organization_store::MockWriteOrganizationStore::new(),
            project_store: crate::ports::project_store::MockWriteProjectStore::new(),
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

    pub fn with_project_store(mut self, store: crate::ports::project_store::MockWriteProjectStore) -> Self {
        self.project_store = store;
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

    fn projects(&self) -> &dyn WriteProjectStore {
        &self.project_store
    }

    async fn save_changes(&mut self) -> Result<(), ApplicationError> {
        self.save_changes_result
            .take()
            .unwrap_or(Ok(()))
    }
}
