use std::sync::Mutex;

use meerkat_application::ports::project_store::WriteProjectStore;
use meerkat_domain::models::project::Project;

pub struct PgWriteProjectStore {
    buffer: Mutex<Vec<Project>>,
}

impl Default for PgWriteProjectStore {
    fn default() -> Self {
        Self::new()
    }
}

impl PgWriteProjectStore {
    pub fn new() -> Self {
        Self {
            buffer: Mutex::new(Vec::new()),
        }
    }

    pub fn take_buffered(&self) -> Vec<Project> {
        let mut guard = self.buffer.lock().unwrap();
        std::mem::take(&mut *guard)
    }
}

impl WriteProjectStore for PgWriteProjectStore {
    fn insert(&self, project: Project) {
        self.buffer.lock().unwrap().push(project);
    }
}
