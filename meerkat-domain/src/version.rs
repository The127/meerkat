#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Version(u64);

impl Version {
    pub fn new(value: u64) -> Self {
        Self(value)
    }

    pub fn initial() -> Self {
        Self(1)
    }

    pub fn as_u64(&self) -> u64 {
        self.0
    }

    pub fn increment(&self) -> Self {
        Self(self.0 + 1)
    }
}