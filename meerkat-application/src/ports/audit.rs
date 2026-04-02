use std::sync::Arc;
use async_trait::async_trait;

use meerkat_domain::models::member::Sub;
use meerkat_domain::models::organization::OrganizationId;
use meerkat_domain::models::permission::EffectivePermission;

#[derive(Debug, Clone)]
pub enum AuditOutcome {
    Allowed,
    Denied,
    Unauthorized,
}

#[derive(Debug, Clone)]
pub struct AuditEvent {
    pub sub: Option<Sub>,
    pub org_id: Option<OrganizationId>,
    pub action: String,
    pub required_permissions: Vec<EffectivePermission>,
    pub outcome: AuditOutcome,
}

#[async_trait]
pub trait AuditLogger: Send + Sync {
    async fn log(&self, event: &AuditEvent);
}

pub struct AuditPipeline {
    loggers: Vec<Arc<dyn AuditLogger>>,
}

impl AuditPipeline {
    pub fn new(loggers: Vec<Arc<dyn AuditLogger>>) -> Self {
        Self { loggers }
    }
}

#[async_trait]
impl AuditLogger for AuditPipeline {
    async fn log(&self, event: &AuditEvent) {
        for logger in &self.loggers {
            logger.log(event).await;
        }
    }
}
