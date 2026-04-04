use std::sync::Arc;
use std::process::Command;

use sqlx::PgPool;
use testcontainers::runners::AsyncRunner;
use testcontainers_modules::postgres::Postgres;
use tokio::net::TcpListener;

use meerkat_api::state::{AppState, AuthState, TenantState};
use meerkat_application::context::{AppContext, RequestContext};
use meerkat_application::error::ApplicationError;
use meerkat_application::mediator::Mediator;
use meerkat_application::behaviors::unit_of_work::UnitOfWorkBehavior;
use meerkat_application::events::EventDispatcher;
use meerkat_application::project_keys::create::{CreateProjectKey, CreateProjectKeyHandler};
use meerkat_application::project_keys::list::{ListProjectKeys, ListProjectKeysHandler};
use meerkat_application::project_keys::on_project_created::GenerateProjectKeyOnProjectCreated;
use meerkat_application::project_keys::revoke::{RevokeProjectKey, RevokeProjectKeyHandler};
use meerkat_application::organizations::create::{CreateOrganization, CreateOrganizationHandler};
use meerkat_application::projects::create::{CreateProject, CreateProjectHandler};
use meerkat_application::ports::error_observer::ErrorPipeline;
use meerkat_application::ports::unit_of_work::UnitOfWorkFactory;
use vec1::vec1;
use meerkat_domain::models::oidc_config::{Audience, ClaimMapping, ClientId, OidcConfig, RoleValues};
use meerkat_domain::models::organization::{Organization, OrganizationSlug};
use meerkat_domain::shared::url::Url;
use meerkat_infrastructure::clock::SystemClock;
use meerkat_infrastructure::jwks::CachedJwksProvider;
use meerkat_infrastructure::oidc_discovery::CachedOidcDiscoveryProvider;
use meerkat_infrastructure::persistence::pg_member_repository::PgMemberRepository;
use meerkat_infrastructure::persistence::pg_oidc_config_read_store::PgOidcConfigReadStore;
use meerkat_infrastructure::persistence::pg_organization_read_store::PgOrganizationReadStore;
use meerkat_infrastructure::persistence::pg_project_read_store::PgProjectReadStore;
use meerkat_infrastructure::persistence::pg_unit_of_work::PgUnitOfWorkFactory;
use meerkat_infrastructure::persistence::pq_health_checker::PgHealthChecker;

fn build_mediator(pool: PgPool) -> Mediator<RequestContext, ApplicationError> {
    let mut mediator = Mediator::new();

    let mut event_dispatcher = EventDispatcher::new();
    event_dispatcher.register(Arc::new(GenerateProjectKeyOnProjectCreated));
    mediator.add_behavior(Arc::new(UnitOfWorkBehavior::new(Arc::new(event_dispatcher))));

    let project_read_store: Arc<dyn meerkat_application::ports::project_read_store::ProjectReadStore> =
        Arc::new(PgProjectReadStore::new(pool.clone()));
    let project_key_read_store: Arc<dyn meerkat_application::ports::project_key_read_store::ProjectKeyReadStore> =
        Arc::new(meerkat_infrastructure::persistence::pg_project_key_read_store::PgProjectKeyReadStore::new(pool));

    mediator.register::<CreateOrganization, _>(CreateOrganizationHandler);
    mediator.register::<CreateProject, _>(CreateProjectHandler);
    mediator.register::<ListProjectKeys, _>(ListProjectKeysHandler::new(project_read_store, project_key_read_store));
    mediator.register::<CreateProjectKey, _>(CreateProjectKeyHandler);
    mediator.register::<RevokeProjectKey, _>(RevokeProjectKeyHandler);
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

    // Bootstrap a master organization so subdomain middleware resolves bare-domain requests
    let uow_factory = PgUnitOfWorkFactory::new(pool.clone(), Arc::new(SystemClock));
    let claim_mapping = ClaimMapping::new(
        "sub", "preferred_username", "roles",
        RoleValues::new(
            vec1!["owner".to_string()],
            vec1!["admin".to_string()],
            vec1!["member".to_string()],
        ),
    ).unwrap();
    let oidc_config = OidcConfig::new(
        "Default".to_string(),
        ClientId::new("test-client").unwrap(),
        Url::new("https://auth.example.com").unwrap(),
        Audience::new("test-api").unwrap(),
        None,
        claim_mapping,
    )
    .unwrap();
    let master_org = Organization::new(
        "Master".to_string(),
        OrganizationSlug::new("master").unwrap(),
        oidc_config,
    )
    .unwrap();
    let mut uow = uow_factory.create().await.unwrap();
    uow.organizations().add(master_org);
    uow.save_changes().await.unwrap();

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let server_port = listener.local_addr().unwrap().port();

    let state = AppState {
        health_checker: Arc::new(PgHealthChecker::new(pool.clone())),
        mediator: Arc::new(build_mediator(pool.clone())),
        context: Arc::new(AppContext::new(
            Arc::new(PgUnitOfWorkFactory::new(pool.clone(), Arc::new(SystemClock))),
            Arc::new(ErrorPipeline::new(vec![])),
        )),
        auth: AuthState {
            oidc_config_read_store: Arc::new(PgOidcConfigReadStore::new(pool.clone())),
            jwks_provider: Arc::new(CachedJwksProvider::new(std::time::Duration::from_secs(300))),
            oidc_discovery_provider: Arc::new(CachedOidcDiscoveryProvider::new(std::time::Duration::from_secs(300))),
            member_repository: Arc::new(PgMemberRepository::new(pool.clone())),
        },
        tenant: TenantState {
            org_read_store: Arc::new(PgOrganizationReadStore::new(pool.clone())),
            base_domain: "127.0.0.1".to_string(),
            master_org_slug: "master".to_string(),
        },
        auth_enabled: false,
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
