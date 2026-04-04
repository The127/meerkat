use meerkat_application::ports::event_repository::EventRepository;
use meerkat_domain::models::event::Event;

use super::change_buffer::ChangeBuffer;

pub(crate) struct EventEntry(pub Event);

pub struct PgEventRepository {
    buffer: ChangeBuffer<EventEntry>,
}

impl Default for PgEventRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl PgEventRepository {
    pub fn new() -> Self {
        Self {
            buffer: ChangeBuffer::new(),
        }
    }

    pub(crate) fn take_entries(&self) -> Vec<EventEntry> {
        self.buffer.take_entries()
    }
}

impl EventRepository for PgEventRepository {
    fn add(&self, event: Event) {
        self.buffer.push(EventEntry(event));
    }
}
