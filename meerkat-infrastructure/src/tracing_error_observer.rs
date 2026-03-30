use async_trait::async_trait;
use meerkat_application::ports::error_observer::{ErrorObserver, ErrorReport, ErrorSeverity};

pub struct TracingErrorObserver;

#[async_trait]
impl ErrorObserver for TracingErrorObserver {
    async fn observe(&self, report: &ErrorReport) {
        match report.severity {
            ErrorSeverity::Warning => {
                tracing::warn!(
                    source = %report.source,
                    "{}",
                    report.message,
                );
            }
            ErrorSeverity::Error => {
                tracing::error!(
                    source = %report.source,
                    "{}",
                    report.message,
                );
            }
            ErrorSeverity::Critical => {
                tracing::error!(
                    source = %report.source,
                    severity = "CRITICAL",
                    "{}",
                    report.message,
                );
            }
        }
    }
}
