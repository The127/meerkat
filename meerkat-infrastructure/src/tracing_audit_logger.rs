use async_trait::async_trait;
use meerkat_application::ports::audit::{AuditEvent, AuditLogger, AuditOutcome};

pub struct TracingAuditLogger;

#[async_trait]
impl AuditLogger for TracingAuditLogger {
    async fn log(&self, event: &AuditEvent) {
        let sub = event.sub.as_ref().map(|s| s.as_str()).unwrap_or("anonymous");
        let org = event.org_id.as_ref().map(|id| id.as_uuid().to_string()).unwrap_or_default();

        match event.outcome {
            AuditOutcome::Allowed => {
                tracing::info!(
                    sub = sub,
                    org_id = %org,
                    action = %event.action,
                    outcome = "allowed",
                    "audit"
                );
            }
            AuditOutcome::Denied => {
                tracing::warn!(
                    sub = sub,
                    org_id = %org,
                    action = %event.action,
                    outcome = "denied",
                    "audit"
                );
            }
            AuditOutcome::Unauthorized => {
                tracing::warn!(
                    action = %event.action,
                    outcome = "unauthorized",
                    "audit"
                );
            }
        }
    }
}
