#[cfg_attr(any(test, feature = "test-utils"), mockall::automock)]
pub trait FingerprintService: Send + Sync {
    fn compute(&self, exception_type: Option<String>, exception_value: Option<String>, message: String) -> String;
}
