use meerkat_domain::models::project_role::ProjectRole;

pub trait ProjectRoleRepository: Send + Sync {
    fn add(&self, role: ProjectRole);
}

#[cfg(any(test, feature = "test-utils"))]
pub struct NoOpProjectRoleRepository;

#[cfg(any(test, feature = "test-utils"))]
impl ProjectRoleRepository for NoOpProjectRoleRepository {
    fn add(&self, _role: ProjectRole) {}
}
