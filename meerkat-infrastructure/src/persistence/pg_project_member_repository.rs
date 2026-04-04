use meerkat_application::ports::project_member_repository::ProjectMemberRepository;
use meerkat_domain::models::project_member::ProjectMember;

use super::change_buffer::ChangeBuffer;

pub(crate) struct ProjectMemberEntry(pub ProjectMember);

pub struct PgProjectMemberRepository {
    buffer: ChangeBuffer<ProjectMemberEntry>,
}

impl Default for PgProjectMemberRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl PgProjectMemberRepository {
    pub fn new() -> Self {
        Self {
            buffer: ChangeBuffer::new(),
        }
    }

    pub(crate) fn take_entries(&self) -> Vec<ProjectMemberEntry> {
        self.buffer.take_entries()
    }
}

impl ProjectMemberRepository for PgProjectMemberRepository {
    fn add(&self, member: ProjectMember) {
        self.buffer.push(ProjectMemberEntry(member));
    }
}
