use std::sync::Mutex;

use meerkat_application::ports::organization_store::WriteOrganizationStore;
use meerkat_domain::models::organization::Organization;

pub struct PgWriteOrganizationStore {
    buffer: Mutex<Vec<Organization>>,
}

impl Default for PgWriteOrganizationStore {
    fn default() -> Self {
        Self::new()
    }
}

impl PgWriteOrganizationStore {
    pub fn new() -> Self {
        Self {
            buffer: Mutex::new(Vec::new()),
        }
    }

    pub fn take_buffered(&self) -> Vec<Organization> {
        let mut guard = self.buffer.lock().unwrap();
        std::mem::take(&mut *guard)
    }
}

impl WriteOrganizationStore for PgWriteOrganizationStore {
    fn insert(&self, org: Organization) {
        self.buffer.lock().unwrap().push(org);
    }
}
