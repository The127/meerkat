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

    pub fn find_entry<F, R>(&self, f: F) -> Option<R>
    where
        F: Fn(&E) -> Option<R>,
    {
        let buf = self.buffer.lock().unwrap();
        for entry in buf.iter() {
            if let Some(result) = f(entry) {
                return Some(result);
            }
        }
        None
    }

    pub fn replace_entry<F>(&self, f: F) -> bool
    where
        F: Fn(&mut E) -> bool,
    {
        let mut buf = self.buffer.lock().unwrap();
        for entry in buf.iter_mut() {
            if f(entry) {
                return true;
            }
        }
        false
    }
}

impl<E> Default for ChangeBuffer<E> {
    fn default() -> Self {
        Self::new()
    }
}

pub(crate) trait BufferEntry<Id, T> {
    fn id(&self) -> &Id;
    fn update_entity(&mut self, entity: T);
    fn make_modified(entity: T, snapshot: T) -> Self;
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

    pub fn find_entry<F, R>(&self, f: F) -> Option<R>
    where
        F: Fn(&E) -> Option<R>,
    {
        self.buffer.find_entry(f)
    }

    pub fn save(&self, id: Id, entity: T)
    where
        E: BufferEntry<Id, T>,
        T: Clone,
        Id: Clone,
    {
        let snapshot = self.take_snapshot(&id);

        let replaced = self.buffer.replace_entry(|entry| {
            if entry.id() == &id {
                entry.update_entity(entity.clone());
                true
            } else {
                false
            }
        });

        if !replaced {
            self.buffer.push(E::make_modified(entity, snapshot));
        }
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

    // --- save merging + find_in_buffer tests ---

    #[derive(Debug, Clone, PartialEq)]
    struct TestEntity {
        id: u32,
        value: String,
    }

    #[derive(Debug)]
    enum TestEntry {
        Added(TestEntity),
        Modified { entity: TestEntity, snapshot: TestEntity },
    }

    impl BufferEntry<u32, TestEntity> for TestEntry {
        fn id(&self) -> &u32 {
            match self {
                TestEntry::Added(e) => &e.id,
                TestEntry::Modified { entity, .. } => &entity.id,
            }
        }

        fn update_entity(&mut self, new: TestEntity) {
            match self {
                TestEntry::Added(e) => *e = new,
                TestEntry::Modified { entity, .. } => *entity = new,
            }
        }

        fn make_modified(entity: TestEntity, snapshot: TestEntity) -> Self {
            TestEntry::Modified { entity, snapshot }
        }
    }

    fn test_tracker() -> ChangeTracker<u32, TestEntity, TestEntry> {
        ChangeTracker::new()
    }

    fn entity(id: u32, value: &str) -> TestEntity {
        TestEntity { id, value: value.to_string() }
    }

    #[test]
    fn given_no_existing_entry_then_save_pushes_modified() {
        // arrange
        let tracker = test_tracker();
        let e = entity(1, "original");
        tracker.track(1, e.clone());

        // act
        let updated = entity(1, "updated");
        tracker.save(1, updated.clone());

        // assert
        let entries = tracker.take_entries();
        assert_eq!(entries.len(), 1);
        match &entries[0] {
            TestEntry::Modified { entity, snapshot } => {
                assert_eq!(entity.value, "updated");
                assert_eq!(snapshot.value, "original");
            }
            _ => panic!("expected Modified"),
        }
    }

    #[test]
    fn given_existing_modified_entry_then_save_replaces_entity_in_place() {
        // arrange
        let tracker = test_tracker();
        let e = entity(1, "v1");
        tracker.track(1, e.clone());
        tracker.save(1, entity(1, "v2"));

        // second save
        tracker.track(1, entity(1, "v2"));

        // act
        tracker.save(1, entity(1, "v3"));

        // assert — should still be one entry, with v3 entity and v1 snapshot
        let entries = tracker.take_entries();
        assert_eq!(entries.len(), 1);
        match &entries[0] {
            TestEntry::Modified { entity, snapshot } => {
                assert_eq!(entity.value, "v3");
                assert_eq!(snapshot.value, "v1");
            }
            _ => panic!("expected Modified"),
        }
    }

    #[test]
    fn given_existing_added_entry_then_save_updates_added_entity() {
        // arrange
        let tracker = test_tracker();
        tracker.push(TestEntry::Added(entity(1, "v1")));
        tracker.track(1, entity(1, "v1"));

        // act
        tracker.save(1, entity(1, "v2"));

        // assert — Added entry should be updated in place, not a new Modified
        let entries = tracker.take_entries();
        assert_eq!(entries.len(), 1);
        match &entries[0] {
            TestEntry::Added(e) => assert_eq!(e.value, "v2"),
            _ => panic!("expected Added with updated value"),
        }
    }

    #[test]
    fn given_different_ids_then_save_does_not_merge() {
        // arrange
        let tracker = test_tracker();
        tracker.push(TestEntry::Added(entity(1, "first")));
        tracker.track(2, entity(2, "second"));

        // act
        tracker.save(2, entity(2, "updated"));

        // assert — two separate entries
        let entries = tracker.take_entries();
        assert_eq!(entries.len(), 2);
    }

    #[test]
    fn given_entry_in_buffer_then_find_entry_returns_it() {
        // arrange
        let tracker = test_tracker();
        tracker.push(TestEntry::Added(entity(1, "hello")));

        // act
        let found = tracker.find_entry(|entry| {
            let e = match entry {
                TestEntry::Added(e) | TestEntry::Modified { entity: e, .. } => e,
            };
            if e.id == 1 { Some(e.clone()) } else { None }
        });

        // assert
        assert_eq!(found.unwrap().value, "hello");
    }

    #[test]
    fn given_no_matching_entry_then_find_entry_returns_none() {
        // arrange
        let tracker = test_tracker();
        tracker.push(TestEntry::Added(entity(1, "hello")));

        // act
        let found = tracker.find_entry(|entry| {
            let e = match entry {
                TestEntry::Added(e) | TestEntry::Modified { entity: e, .. } => e,
            };
            if e.id == 99 { Some(e.clone()) } else { None }
        });

        // assert
        assert!(found.is_none());
    }

    #[test]
    fn given_find_then_save_then_save_then_one_modified_with_original_snapshot() {
        // arrange — simulates: load from DB, mutate, save, mutate again, save again
        let tracker = test_tracker();
        tracker.track(1, entity(1, "from_db"));

        // first save
        tracker.save(1, entity(1, "v2"));

        // second find+save (like domain event handler)
        tracker.track(1, entity(1, "v2"));
        tracker.save(1, entity(1, "v3"));

        // assert — one entry, latest entity, original snapshot
        let entries = tracker.take_entries();
        assert_eq!(entries.len(), 1);
        match &entries[0] {
            TestEntry::Modified { entity, snapshot } => {
                assert_eq!(entity.value, "v3");
                assert_eq!(snapshot.value, "from_db");
            }
            _ => panic!("expected Modified"),
        }
    }

    #[test]
    fn given_add_then_save_then_one_added_with_latest_entity() {
        // arrange — simulates: create entity, then mutate it in same UoW
        let tracker = test_tracker();
        tracker.push(TestEntry::Added(entity(1, "created")));
        tracker.track(1, entity(1, "created"));

        // act
        tracker.save(1, entity(1, "mutated"));

        // assert — stays Added (not Modified) since it's new
        let entries = tracker.take_entries();
        assert_eq!(entries.len(), 1);
        match &entries[0] {
            TestEntry::Added(e) => assert_eq!(e.value, "mutated"),
            _ => panic!("expected Added with updated value"),
        }
    }

    #[test]
    fn given_add_then_save_twice_then_one_added_with_latest_entity() {
        // arrange
        let tracker = test_tracker();
        tracker.push(TestEntry::Added(entity(1, "created")));
        tracker.track(1, entity(1, "created"));
        tracker.save(1, entity(1, "v2"));
        tracker.track(1, entity(1, "v2"));

        // act
        tracker.save(1, entity(1, "v3"));

        // assert — still one Added entry
        let entries = tracker.take_entries();
        assert_eq!(entries.len(), 1);
        match &entries[0] {
            TestEntry::Added(e) => assert_eq!(e.value, "v3"),
            _ => panic!("expected Added"),
        }
    }

    #[test]
    fn given_multiple_entities_then_save_only_merges_matching_id() {
        // arrange
        let tracker = test_tracker();
        tracker.push(TestEntry::Added(entity(1, "first")));
        tracker.push(TestEntry::Added(entity(2, "second")));
        tracker.track(1, entity(1, "first"));

        // act
        tracker.save(1, entity(1, "updated"));

        // assert
        let entries = tracker.take_entries();
        assert_eq!(entries.len(), 2);
        match &entries[0] {
            TestEntry::Added(e) => assert_eq!(e.value, "updated"),
            _ => panic!("expected Added"),
        }
        match &entries[1] {
            TestEntry::Added(e) => assert_eq!(e.value, "second"),
            _ => panic!("expected Added"),
        }
    }

    #[test]
    fn given_find_entry_after_save_then_returns_latest_entity() {
        // arrange
        let tracker = test_tracker();
        tracker.track(1, entity(1, "from_db"));
        tracker.save(1, entity(1, "saved"));

        // act
        let found = tracker.find_entry(|entry| {
            let e = match entry {
                TestEntry::Added(e) | TestEntry::Modified { entity: e, .. } => e,
            };
            if e.id == 1 { Some(e.clone()) } else { None }
        });

        // assert
        assert_eq!(found.unwrap().value, "saved");
    }

    // --- Deleted entry tests ---

    #[derive(Debug)]
    enum DeletableTestEntry {
        Added(TestEntity),
        Modified { entity: TestEntity, snapshot: TestEntity },
        Deleted(u32),
    }

    impl BufferEntry<u32, TestEntity> for DeletableTestEntry {
        fn id(&self) -> &u32 {
            match self {
                DeletableTestEntry::Added(e) => &e.id,
                DeletableTestEntry::Modified { entity, .. } => &entity.id,
                DeletableTestEntry::Deleted(id) => id,
            }
        }

        fn update_entity(&mut self, new: TestEntity) {
            match self {
                DeletableTestEntry::Added(e) => *e = new,
                DeletableTestEntry::Modified { entity, .. } => *entity = new,
                DeletableTestEntry::Deleted(_) => panic!("cannot update a deleted entity"),
            }
        }

        fn make_modified(entity: TestEntity, snapshot: TestEntity) -> Self {
            DeletableTestEntry::Modified { entity, snapshot }
        }
    }

    #[test]
    #[should_panic(expected = "cannot update a deleted entity")]
    fn given_deleted_entry_then_update_entity_panics() {
        // arrange
        let mut entry = DeletableTestEntry::Deleted(1);

        // act
        entry.update_entity(entity(1, "new"));
    }
}
