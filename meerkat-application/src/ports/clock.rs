use chrono::{DateTime, Utc};

pub trait Clock: Send + Sync {
    fn now(&self) -> DateTime<Utc>;
}

#[cfg(any(test, feature = "test-utils"))]
pub struct MockClock {
    now: std::sync::Arc<parking_lot::RwLock<DateTime<Utc>>>,
}

#[cfg(any(test, feature = "test-utils"))]
impl MockClock {
    pub fn new(now: DateTime<Utc>) -> Self {
        Self {
            now: std::sync::Arc::new(parking_lot::RwLock::new(now)),
        }
    }

    pub fn set_now(&self, now: DateTime<Utc>) {
        let mut guard = self.now.write();
        *guard = now;
    }

    pub fn advance(&self, duration: chrono::Duration) {
        let mut guard = self.now.write();
        *guard = *guard + duration;
    }
}

#[cfg(any(test, feature = "test-utils"))]
impl Clock for MockClock {
    fn now(&self) -> DateTime<Utc> {
        let guard = self.now.read();
        *guard
    }
}
