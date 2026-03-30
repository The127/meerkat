use meerkat_domain::models::organization::Organization;

#[cfg_attr(any(test, feature = "test-utils"), mockall::automock)]
pub trait WriteOrganizationStore: Send + Sync {
    fn insert(&self, org: Organization);
}
