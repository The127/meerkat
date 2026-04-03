pub mod config;
mod bootstrap;

use std::sync::Arc;
use anyhow::Context;
use clap::{Parser, Subcommand};
use sqlx::PgPool;
use tokio::sync::watch;
use tracing::info;
use meerkat_api::state::AppState;
use meerkat_application::context::{AppContext, RequestContext};
use meerkat_application::error::ApplicationError;
use meerkat_application::mediator::Mediator;
use meerkat_application::behaviors::authorization::AuthorizationBehavior;
use meerkat_application::ports::audit::AuditPipeline;
use meerkat_application::behaviors::unit_of_work::UnitOfWorkBehavior;
use meerkat_application::organizations::create::{CreateOrganization, CreateOrganizationHandler};
use meerkat_application::organizations::delete::{DeleteOrganization, DeleteOrganizationHandler};
use meerkat_application::organizations::get::{GetOrganization, GetOrganizationHandler};
use meerkat_application::organizations::rename::{RenameOrganization, RenameOrganizationHandler};
use meerkat_application::projects::create::{CreateProject, CreateProjectHandler};
use meerkat_application::projects::delete::{DeleteProject, DeleteProjectHandler};
use meerkat_application::projects::get::{GetProject, GetProjectHandler};
use meerkat_application::projects::list::{ListProjects, ListProjectsHandler};
use meerkat_application::projects::rename::{RenameProject, RenameProjectHandler};
use meerkat_application::ports::error_observer::ErrorPipeline;
use meerkat_infrastructure::clock::SystemClock;
use meerkat_infrastructure::persistence::pg_unit_of_work::PgUnitOfWorkFactory;
use meerkat_infrastructure::persistence::pq_health_checker::PgHealthChecker;
use meerkat_infrastructure::jwks::CachedJwksProvider;
use meerkat_infrastructure::oidc_discovery::CachedOidcDiscoveryProvider;
use meerkat_infrastructure::persistence::pg_member_repository::PgMemberRepository;
use meerkat_infrastructure::persistence::pg_oidc_config_read_store::PgOidcConfigReadStore;
use meerkat_infrastructure::persistence::pg_project_permission_read_store::PgProjectPermissionReadStore;
use meerkat_infrastructure::persistence::pg_organization_read_store::PgOrganizationReadStore;
use meerkat_infrastructure::persistence::pg_project_read_store::PgProjectReadStore;
use meerkat_infrastructure::tracing_audit_logger::TracingAuditLogger;
use meerkat_infrastructure::tracing_error_observer::TracingErrorObserver;
use crate::config::MeerkatConfig;

#[derive(Debug, Parser)]
#[command(name = "meerkat", about = "Meerkat error reporting service")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Run only the API server
    Api,
    /// Run database migrations
    Migrate,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let cli = Cli::parse();
    let config = MeerkatConfig::from_env()?;

    match cli.command {
        Commands::Migrate => {
            let pool = create_pool(&config).await?;
            info!("Running database migrations...");
            sqlx::migrate!()
                .run(&pool)
                .await
                .context("Failed to run database migrations")?;
            info!("Migrations complete.");
        }
        Commands::Api => {
            let pool = create_pool(&config).await?;

            bootstrap::bootstrap_master(&config, &pool, &SystemClock).await?;

            let (shutdown_tx, shutdown_rx) = watch::channel(false);

            let api_handle = tokio::spawn(async move {
                run_api(pool, &config, shutdown_rx).await
            });

            tokio::signal::ctrl_c()
                .await
                .context("Failed to listen for ctrl-c")?;
            info!("Shutdown signal received");
            let _ = shutdown_tx.send(true);

            api_handle.await??;
        }
    }


    Ok(())
}

async fn create_pool(config: &MeerkatConfig) -> anyhow::Result<PgPool> {
    PgPool::connect(&config.database_url)
        .await
        .context("Failed to connect to database")
}

fn build_mediator(
    audit_logger: Arc<dyn meerkat_application::ports::audit::AuditLogger>,
    project_permission_store: Arc<dyn meerkat_application::ports::project_permission_read_store::ProjectPermissionReadStore>,
    org_read_store: Arc<dyn meerkat_application::ports::organization_read_store::OrganizationReadStore>,
    project_read_store: Arc<dyn meerkat_application::ports::project_read_store::ProjectReadStore>,
) -> Mediator<RequestContext, ApplicationError> {
    let mut mediator = Mediator::new();
    mediator.add_behavior(Arc::new(AuthorizationBehavior::new(audit_logger, project_permission_store)));
    mediator.add_behavior(Arc::new(UnitOfWorkBehavior));
    mediator.register::<CreateOrganization, _>(CreateOrganizationHandler);
    mediator.register::<RenameOrganization, _>(RenameOrganizationHandler);
    mediator.register::<DeleteOrganization, _>(DeleteOrganizationHandler);
    mediator.register::<GetOrganization, _>(GetOrganizationHandler::new(org_read_store));
    mediator.register::<CreateProject, _>(CreateProjectHandler);
    mediator.register::<RenameProject, _>(RenameProjectHandler);
    mediator.register::<DeleteProject, _>(DeleteProjectHandler);
    mediator.register::<GetProject, _>(GetProjectHandler::new(project_read_store.clone()));
    mediator.register::<ListProjects, _>(ListProjectsHandler::new(project_read_store));
    mediator
}

async fn run_api(
    pool: PgPool,
    config: &MeerkatConfig,
    mut shutdown: watch::Receiver<bool>,
) -> anyhow::Result<()> {
    let health_checker = Arc::new(PgHealthChecker::new(pool.clone()));

    let uow_factory = Arc::new(PgUnitOfWorkFactory::new(pool.clone()));

    let error_observer = Arc::new(ErrorPipeline::new(vec![
        Arc::new(TracingErrorObserver),
    ]));

    let context = Arc::new(AppContext::new(
        Arc::new(SystemClock),
        uow_factory,
        error_observer,
    ));

    let org_read_store = Arc::new(PgOrganizationReadStore::new(pool.clone()));
    let project_read_store = Arc::new(PgProjectReadStore::new(pool.clone()));

    let audit_logger: Arc<dyn meerkat_application::ports::audit::AuditLogger> = Arc::new(AuditPipeline::new(vec![
        Arc::new(TracingAuditLogger),
    ]));
    let project_permission_store: Arc<dyn meerkat_application::ports::project_permission_read_store::ProjectPermissionReadStore> =
        Arc::new(PgProjectPermissionReadStore::new(pool.clone()));
    let mediator = Arc::new(build_mediator(audit_logger, project_permission_store, org_read_store.clone(), project_read_store.clone()));
    let oidc_config_read_store = Arc::new(PgOidcConfigReadStore::new(pool.clone()));
    let jwks_provider = Arc::new(CachedJwksProvider::new(std::time::Duration::from_secs(300)));
    let member_repository = Arc::new(PgMemberRepository::new(pool.clone()));
    let oidc_discovery_provider = Arc::new(CachedOidcDiscoveryProvider::new(std::time::Duration::from_secs(300)));

    let state = AppState {
        health_checker,
        mediator,
        context,
        org_read_store,
        project_read_store,
        oidc_config_read_store,
        jwks_provider,
        member_repository,
        oidc_discovery_provider,
        base_domain: config.base_domain.clone(),
        master_org_slug: config.master_org_slug.clone(),
        auth_enabled: true,
    };

    let router = meerkat_api::router(state);

    let listener = tokio::net::TcpListener::bind(&config.listen_addr)
        .await
        .with_context(|| format!("Failed to bind to {}", config.listen_addr))?;

    info!("Listening on {}", config.listen_addr);

    axum::serve(listener, router)
        .with_graceful_shutdown(async move {
            let _ = shutdown.changed().await;
        })
        .await
        .context("Server error")?;

    Ok(())
}
