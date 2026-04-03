use std::sync::Mutex;

use meerkat_application::ports::project_role_repository::ProjectRoleRepository;
use meerkat_domain::models::project_role::ProjectRole;

pub(crate) struct ProjectRoleEntry(pub ProjectRole);

#[derive(Default)]
pub struct PgProjectRoleRepository {
    buffer: Mutex<Vec<ProjectRoleEntry>>,
}

impl PgProjectRoleRepository {
    pub fn new() -> Self {
        Self {
            buffer: Mutex::new(Vec::new()),
        }
    }

    pub(crate) fn take_entries(&self) -> Vec<ProjectRoleEntry> {
        std::mem::take(&mut *self.buffer.lock().unwrap())
    }
}

impl ProjectRoleRepository for PgProjectRoleRepository {
    fn add(&self, role: ProjectRole) {
        self.buffer.lock().unwrap().push(ProjectRoleEntry(role));
    }
}
