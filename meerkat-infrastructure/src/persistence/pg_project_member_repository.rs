use std::sync::Mutex;

use meerkat_application::ports::project_member_repository::ProjectMemberRepository;
use meerkat_domain::models::project_member::ProjectMember;

pub(crate) struct ProjectMemberEntry(pub ProjectMember);

#[derive(Default)]
pub struct PgProjectMemberRepository {
    buffer: Mutex<Vec<ProjectMemberEntry>>,
}

impl PgProjectMemberRepository {
    pub fn new() -> Self {
        Self {
            buffer: Mutex::new(Vec::new()),
        }
    }

    pub(crate) fn take_entries(&self) -> Vec<ProjectMemberEntry> {
        std::mem::take(&mut *self.buffer.lock().unwrap())
    }
}

impl ProjectMemberRepository for PgProjectMemberRepository {
    fn add(&self, member: ProjectMember) {
        self.buffer.lock().unwrap().push(ProjectMemberEntry(member));
    }
}
