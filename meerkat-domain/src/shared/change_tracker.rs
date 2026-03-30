#[derive(Debug, Clone)]
pub struct ChangeTracker<T> {
    changes: Vec<T>,
}

impl<T> ChangeTracker<T> {
    pub fn new() -> Self {
        Self {
            changes: Vec::new(),
        }
    }

    pub fn record(&mut self, change: T) {
        self.changes.push(change);
    }

    pub fn pull_changes(&mut self) -> Vec<T> {
        std::mem::take(&mut self.changes)
    }

    pub fn has_changes(&self) -> bool {
        !self.changes.is_empty()
    }

    pub fn changes_count(&self) -> usize {
        self.changes.len()
    }
}

impl<T> Default for ChangeTracker<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn given_a_new_change_tracker_it_should_be_empty() {
        // arrange
        let tracker: ChangeTracker<String> = ChangeTracker::new();

        // act & assert
        assert!(!tracker.has_changes());
        assert_eq!(tracker.changes_count(), 0);
    }

    #[test]
    fn given_a_change_tracker_when_recording_a_change_it_should_no_longer_be_empty() {
        // arrange
        let mut tracker = ChangeTracker::new();
        let change = "test change".to_string();

        // act
        tracker.record(change);

        // assert
        assert!(tracker.has_changes());
        assert_eq!(tracker.changes_count(), 1);
    }

    #[test]
    fn given_a_change_tracker_with_changes_when_pulling_changes_it_should_return_all_changes_and_clear_itself() {
        // arrange
        let mut tracker = ChangeTracker::new();
        tracker.record("change 1".to_string());
        tracker.record("change 2".to_string());

        // act
        let changes = tracker.pull_changes();

        // assert
        assert_eq!(changes.len(), 2);
        assert_eq!(changes[0], "change 1");
        assert_eq!(changes[1], "change 2");
        assert!(!tracker.has_changes());
        assert_eq!(tracker.changes_count(), 0);
    }

    #[test]
    fn given_default_change_tracker_it_should_be_initialized_identically_to_new() {
        // arrange & act
        let tracker: ChangeTracker<i32> = ChangeTracker::default();

        // assert
        assert!(!tracker.has_changes());
        assert_eq!(tracker.changes_count(), 0);
    }
}
