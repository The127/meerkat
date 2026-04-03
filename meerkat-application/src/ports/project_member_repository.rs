use meerkat_domain::models::project_member::ProjectMember;

pub trait ProjectMemberRepository: Send + Sync {
    fn add(&self, member: ProjectMember);
}

#[cfg(any(test, feature = "test-utils"))]
pub struct NoOpProjectMemberRepository;

#[cfg(any(test, feature = "test-utils"))]
impl ProjectMemberRepository for NoOpProjectMemberRepository {
    fn add(&self, _member: ProjectMember) {}
}
