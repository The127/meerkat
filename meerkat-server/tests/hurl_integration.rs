use std::sync::Arc;
use std::process::Command;

use sqlx::PgPool;
use testcontainers::runners::AsyncRunner;
use testcontainers_modules::postgres::Postgres;
use tokio::net::TcpListener;

use meerkat_api::state::AppState;
use meerkat_application::context::{AppContext, RequestContext};
use meerkat_application::error::ApplicationError;
use meerkat_application::mediator::Mediator;
use meerkat_application::behaviors::unit_of_work::UnitOfWorkBehavior;
use meerkat_application::organizations::create::{CreateOrganization, CreateOrganizationHandler};
use meerkat_application::projects::create::{CreateProject, CreateProjectHandler};
use meerkat_application::ports::error_observer::ErrorPipeline;
use meerkat_infrastructure::clock::SystemClock;
use meerkat_infrastructure::persistence::pg_unit_of_work::PgUnitOfWorkFactory;
use meerkat_infrastructure::persistence::pq_health_checker::PgHealthChecker;

fn build_mediator() -> Mediator<RequestContext, ApplicationError> {
    let mut mediator = Mediator::new();
    mediator.add_behavior(Arc::new(UnitOfWorkBehavior));
    mediator.register::<CreateOrganization, _>(CreateOrganizationHandler);
    mediator.register::<CreateProject, _>(CreateProjectHandler);
    mediator
}

#[tokio::test(flavor = "multi_thread")]
async fn hurl_integration_tests() {
    let container = Postgres::default().start().await.unwrap();
    let host = container.get_host().await.unwrap();
    let port = container.get_host_port_ipv4(5432).await.unwrap();
    let db_url = format!("postgres://postgres:postgres@{}:{}/postgres", host, port);

    let pool = PgPool::connect(&db_url).await.unwrap();
    sqlx::migrate!().run(&pool).await.unwrap();

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let server_port = listener.local_addr().unwrap().port();

    let state = AppState {
        health_checker: Arc::new(PgHealthChecker::new(pool.clone())),
        mediator: Arc::new(build_mediator()),
        context: Arc::new(AppContext::new(
            Arc::new(SystemClock),
            Arc::new(PgUnitOfWorkFactory::new(pool)),
            Arc::new(ErrorPipeline::new(vec![])),
        )),
    };

    let router = meerkat_api::router(state);

    let server = tokio::spawn(async move {
        axum::serve(listener, router).await.unwrap();
    });

    let server_host = format!("http://127.0.0.1:{}", server_port);

    let workspace_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    let pattern = workspace_root.join("meerkat-api/src/handlers/*.hurl");
    let hurl_files: Vec<_> = glob::glob(pattern.to_str().unwrap())
        .expect("failed to glob hurl files")
        .filter_map(|e| e.ok())
        .collect();

    assert!(!hurl_files.is_empty(), "no hurl files found");

    let output = Command::new("hurl")
        .arg("--test")
        .arg("--jobs")
        .arg("1")
        .arg("--variable")
        .arg(format!("host={}", server_host))
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
