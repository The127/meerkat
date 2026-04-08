use chrono::{DateTime, Utc};
use raccoon_clock_rs::Clock;

pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> DateTime<Utc> {
        Utc::now()
    }
}
