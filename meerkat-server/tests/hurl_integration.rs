use std::sync::Arc;
use std::process::Command;
use tokio::net::TcpListener;

use meerkat_api::state::AppState;
use meerkat_application::context::AppContext;
use meerkat_application::error::ApplicationError;
use meerkat_application::mediator::Mediator;
use meerkat_application::organizations::create::{CreateOrganization, CreateOrganizationHandler};
use meerkat_application::ports::error_observer::ErrorPipeline;
use meerkat_application::ports::unit_of_work::UnitOfWorkFactory;
use meerkat_domain::ports::clock::MockClock;

struct AlwaysHealthy;

#[async_trait::async_trait]
impl meerkat_application::ports::health::HealthChecker for AlwaysHealthy {
    async fn check(&self) -> bool {
        true
    }
}

struct TodoUnitOfWorkFactory;

#[async_trait::async_trait]
impl UnitOfWorkFactory for TodoUnitOfWorkFactory {
    async fn create(&self) -> Result<Box<dyn meerkat_application::ports::unit_of_work::UnitOfWork>, ApplicationError> {
        Err(ApplicationError::Internal("UnitOfWork not yet implemented".to_string()))
    }
}

fn build_mediator() -> Mediator<AppContext, ApplicationError> {
    let mut mediator = Mediator::new();
    mediator.register::<CreateOrganization, _>(CreateOrganizationHandler);
    mediator
}

#[tokio::test(flavor = "multi_thread")]
async fn hurl_integration_tests() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();

    let state = AppState {
        health_checker: Arc::new(AlwaysHealthy),
        mediator: Arc::new(build_mediator()),
        context: Arc::new(AppContext {
            clock: Arc::new(MockClock::new(chrono::Utc::now())),
            uow_factory: Arc::new(TodoUnitOfWorkFactory),
            error_observer: Arc::new(ErrorPipeline::new(vec![])),
        }),
    };

    let router = meerkat_api::router(state);

    let server = tokio::spawn(async move {
        axum::serve(listener, router).await.unwrap();
    });

    let host = format!("http://127.0.0.1:{}", port);

    let workspace_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    let pattern = workspace_root.join("meerkat-api/src/handlers/*.hurl");
    let hurl_files: Vec<_> = glob::glob(pattern.to_str().unwrap())
        .expect("failed to glob hurl files")
        .filter_map(|e| e.ok())
        .collect();

    assert!(!hurl_files.is_empty(), "no hurl files found");

    let output = Command::new("hurl")
        .arg("--test")
        .arg("--variable")
        .arg(format!("host={}", host))
        .args(&hurl_files)
        .output()
        .expect("failed to run hurl - is it installed?");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    server.abort();

    assert!(
        output.status.success(),
        "hurl tests failed:\nstdout:\n{}\nstderr:\n{}",
        stdout,
        stderr,
    );
}
