use std::collections::HashMap;
use std::hash::Hash;
use std::sync::Mutex;

pub(crate) struct ChangeBuffer<E> {
    buffer: Mutex<Vec<E>>,
}

impl<E> ChangeBuffer<E> {
    pub fn new() -> Self {
        Self {
            buffer: Mutex::new(Vec::new()),
        }
    }

    pub fn push(&self, entry: E) {
        self.buffer.lock().unwrap().push(entry);
    }

    pub fn take_entries(&self) -> Vec<E> {
        std::mem::take(&mut *self.buffer.lock().unwrap())
    }
}

impl<E> Default for ChangeBuffer<E> {
    fn default() -> Self {
        Self::new()
    }
}

pub(crate) struct ChangeTracker<Id, T, E> {
    snapshots: Mutex<HashMap<Id, T>>,
    buffer: ChangeBuffer<E>,
}

impl<Id, T, E> ChangeTracker<Id, T, E>
where
    Id: Eq + Hash,
{
    pub fn new() -> Self {
        Self {
            snapshots: Mutex::new(HashMap::new()),
            buffer: ChangeBuffer::new(),
        }
    }

    pub fn push(&self, entry: E) {
        self.buffer.push(entry);
    }

    pub fn take_entries(&self) -> Vec<E> {
        self.buffer.take_entries()
    }

    pub fn track(&self, id: Id, snapshot: T) {
        self.snapshots.lock().unwrap().insert(id, snapshot);
    }

    pub fn take_snapshot(&self, id: &Id) -> T {
        self.snapshots
            .lock()
            .unwrap()
            .remove(id)
            .expect("save called without prior find")
    }

    pub fn remove_snapshot(&self, id: &Id) {
        self.snapshots.lock().unwrap().remove(id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn given_empty_buffer_then_take_entries_returns_empty() {
        // arrange
        let buffer = ChangeBuffer::<String>::new();

        // act
        let entries = buffer.take_entries();

        // assert
        assert!(entries.is_empty());
    }

    #[test]
    fn given_pushed_entries_then_take_entries_returns_all() {
        // arrange
        let buffer = ChangeBuffer::new();
        buffer.push("a".to_string());
        buffer.push("b".to_string());

        // act
        let entries = buffer.take_entries();

        // assert
        assert_eq!(entries, vec!["a", "b"]);
    }

    #[test]
    fn given_taken_entries_then_buffer_is_empty() {
        // arrange
        let buffer = ChangeBuffer::new();
        buffer.push("a".to_string());
        let _ = buffer.take_entries();

        // act
        let entries = buffer.take_entries();

        // assert
        assert!(entries.is_empty());
    }

    #[test]
    fn given_tracked_snapshot_then_take_snapshot_returns_it() {
        // arrange
        let tracker = ChangeTracker::<u32, String, String>::new();
        tracker.track(1, "snapshot".to_string());

        // act
        let snapshot = tracker.take_snapshot(&1);

        // assert
        assert_eq!(snapshot, "snapshot");
    }

    #[test]
    fn given_taken_snapshot_then_it_is_removed() {
        // arrange
        let tracker = ChangeTracker::<u32, String, String>::new();
        tracker.track(1, "snapshot".to_string());
        let _ = tracker.take_snapshot(&1);

        // act / assert
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            tracker.take_snapshot(&1);
        }));
        assert!(result.is_err());
    }

    #[test]
    fn given_removed_snapshot_then_take_snapshot_panics() {
        // arrange
        let tracker = ChangeTracker::<u32, String, String>::new();
        tracker.track(1, "snapshot".to_string());
        tracker.remove_snapshot(&1);

        // act / assert
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            tracker.take_snapshot(&1);
        }));
        assert!(result.is_err());
    }

    #[test]
    fn given_tracker_with_entries_then_take_entries_drains_buffer() {
        // arrange
        let tracker = ChangeTracker::<u32, String, String>::new();
        tracker.push("entry1".to_string());
        tracker.push("entry2".to_string());

        // act
        let entries = tracker.take_entries();

        // assert
        assert_eq!(entries, vec!["entry1", "entry2"]);
        assert!(tracker.take_entries().is_empty());
    }

    #[test]
    fn given_multiple_snapshots_then_each_can_be_taken_independently() {
        // arrange
        let tracker = ChangeTracker::<u32, String, String>::new();
        tracker.track(1, "first".to_string());
        tracker.track(2, "second".to_string());

        // act
        let first = tracker.take_snapshot(&1);

        // assert
        assert_eq!(first, "first");
        assert_eq!(tracker.take_snapshot(&2), "second");
    }

    #[test]
    #[should_panic(expected = "save called without prior find")]
    fn given_no_snapshot_then_take_snapshot_panics() {
        // arrange
        let tracker = ChangeTracker::<u32, String, String>::new();

        // act
        tracker.take_snapshot(&1);
    }

    #[test]
    fn given_overwritten_snapshot_then_latest_is_returned() {
        // arrange
        let tracker = ChangeTracker::<u32, String, String>::new();
        tracker.track(1, "old".to_string());
        tracker.track(1, "new".to_string());

        // act
        let snapshot = tracker.take_snapshot(&1);

        // assert
        assert_eq!(snapshot, "new");
    }
}
