use meerkat_domain::models::project::Project;

#[cfg_attr(any(test, feature = "test-utils"), mockall::automock)]
pub trait WriteProjectStore: Send + Sync {
    fn insert(&self, project: Project);
}
