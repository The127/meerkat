use std::sync::Arc;
use async_trait::async_trait;

#[derive(Debug, Clone)]
pub enum ErrorSeverity {
    Warning,
    Error,
    Critical,
}

#[derive(Debug, Clone)]
pub struct ErrorReport {
    pub message: String,
    pub severity: ErrorSeverity,
    pub source: String,
}

#[async_trait]
pub trait ErrorObserver: Send + Sync {
    async fn observe(&self, report: &ErrorReport);
}

pub struct ErrorPipeline {
    observers: Vec<Arc<dyn ErrorObserver>>,
}

impl ErrorPipeline {
    pub fn new(observers: Vec<Arc<dyn ErrorObserver>>) -> Self {
        Self { observers }
    }
}

#[async_trait]
impl ErrorObserver for ErrorPipeline {
    async fn observe(&self, report: &ErrorReport) {
        for observer in &self.observers {
            observer.observe(report).await;
        }
    }
}
