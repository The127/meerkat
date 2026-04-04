use meerkat_application::ports::project_role_repository::ProjectRoleRepository;
use meerkat_domain::models::project_role::ProjectRole;

use super::change_buffer::ChangeBuffer;

pub(crate) struct ProjectRoleEntry(pub ProjectRole);

pub struct PgProjectRoleRepository {
    buffer: ChangeBuffer<ProjectRoleEntry>,
}

impl Default for PgProjectRoleRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl PgProjectRoleRepository {
    pub fn new() -> Self {
        Self {
            buffer: ChangeBuffer::new(),
        }
    }

    pub(crate) fn take_entries(&self) -> Vec<ProjectRoleEntry> {
        self.buffer.take_entries()
    }
}

impl ProjectRoleRepository for PgProjectRoleRepository {
    fn add(&self, role: ProjectRole) {
        self.buffer.push(ProjectRoleEntry(role));
    }
}
