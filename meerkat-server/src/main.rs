pub mod config;
mod bootstrap;

use std::sync::Arc;
use anyhow::Context;
use clap::{Parser, Subcommand};
use sqlx::PgPool;
use tokio::sync::watch;
use tracing::info;
use meerkat_api::state::{AppState, AuthState, TenantState};
use meerkat_application::context::{AppContext, RequestContext};
use meerkat_application::error::ApplicationError;
use meerkat_application::mediator::Mediator;
use meerkat_application::behaviors::authorization::AuthorizationBehavior;
use meerkat_application::behaviors::rate_limit::RateLimitBehavior;
use meerkat_application::ports::audit::AuditPipeline;
use meerkat_application::behaviors::unit_of_work::UnitOfWorkBehavior;
use meerkat_application::events::EventDispatcher;
use meerkat_application::issues::get::{GetIssue, GetIssueHandler};
use meerkat_application::issues::ignore::{IgnoreIssue, IgnoreIssueHandler};
use meerkat_application::issues::list::{ListIssues, ListIssuesHandler};
use meerkat_application::issues::list_events::{ListIssueEvents, ListIssueEventsHandler};
use meerkat_application::issues::on_event_recorded::RegressResolvedIssueOnNewEvent;
use meerkat_application::issues::reopen::{ReopenIssue, ReopenIssueHandler};
use meerkat_application::issues::resolve::{ResolveIssue, ResolveIssueHandler};
use meerkat_application::project_keys::create::{CreateProjectKey, CreateProjectKeyHandler};
use meerkat_application::project_keys::list::{ListProjectKeys, ListProjectKeysHandler};
use meerkat_application::project_keys::on_project_created::GenerateProjectKeyOnProjectCreated;
use meerkat_application::project_keys::revoke::{RevokeProjectKey, RevokeProjectKeyHandler};
use meerkat_application::project_keys::update_rate_limit::{UpdateProjectKeyRateLimit, UpdateProjectKeyRateLimitHandler};
use meerkat_application::organizations::create::{CreateOrganization, CreateOrganizationHandler};
use meerkat_application::organizations::delete::{DeleteOrganization, DeleteOrganizationHandler};
use meerkat_application::organizations::get::{GetOrganization, GetOrganizationHandler};
use meerkat_application::organizations::get_oidc_config::{GetOidcConfig, GetOidcConfigHandler};
use meerkat_application::organizations::activate_oidc_config::{ActivateOidcConfig, ActivateOidcConfigHandler};
use meerkat_application::organizations::add_oidc_config::{AddOidcConfig, AddOidcConfigHandler};
use meerkat_application::organizations::delete_oidc_config::{DeleteOidcConfig, DeleteOidcConfigHandler};
use meerkat_application::organizations::update_oidc_claim_mapping::{UpdateOidcClaimMapping, UpdateOidcClaimMappingHandler};
use meerkat_application::organizations::list_oidc_configs::{ListOidcConfigs, ListOidcConfigsHandler};
use meerkat_application::organizations::rename::{RenameOrganization, RenameOrganizationHandler};
use meerkat_application::projects::create::{CreateProject, CreateProjectHandler};
use meerkat_application::projects::delete::{DeleteProject, DeleteProjectHandler};
use meerkat_application::projects::get::{GetProject, GetProjectHandler};
use meerkat_application::projects::list::{ListProjects, ListProjectsHandler};
use meerkat_application::projects::list_members::{ListProjectMembers, ListProjectMembersHandler};
use meerkat_application::projects::list_roles::{ListProjectRoles, ListProjectRolesHandler};
use meerkat_application::members::get_current_user::{GetCurrentUser, GetCurrentUserHandler};
use meerkat_application::members::get_member_access::{GetMemberAccess, GetMemberAccessHandler};
use meerkat_application::members::list_member_projects::{ListMemberProjects, ListMemberProjectsHandler};
use meerkat_application::members::list_members::{ListMembers, ListMembersHandler};
use meerkat_application::projects::rename::{RenameProject, RenameProjectHandler};
use meerkat_application::ports::error_observer::ErrorPipeline;
use meerkat_infrastructure::clock::SystemClock;
use meerkat_infrastructure::persistence::pg_unit_of_work::PgUnitOfWorkFactory;
use meerkat_infrastructure::persistence::pq_health_checker::PgHealthChecker;
use meerkat_infrastructure::jwks::CachedJwksProvider;
use meerkat_infrastructure::oidc_discovery::CachedOidcDiscoveryProvider;
use meerkat_application::events::ingest::{IngestEvent, IngestEventHandler};
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

    match cli.command {
        Commands::Migrate => {
            let database_url = std::env::var("MEERKAT_DATABASE_URL")
                .context("MEERKAT_DATABASE_URL environment variable must be set")?;
            let pool = PgPool::connect(&database_url)
                .await
                .context("Failed to connect to database")?;
            info!("Running database migrations...");
            sqlx::migrate!()
                .run(&pool)
                .await
                .context("Failed to run database migrations")?;
            info!("Migrations complete.");
        }
        Commands::Api => {
            let config = MeerkatConfig::from_env()?;
            let pool = create_pool(&config).await?;

            bootstrap::bootstrap_master(&config, &pool).await?;

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

struct ReadStores {
    org: Arc<dyn meerkat_application::ports::organization_read_store::OrganizationReadStore>,
    oidc_config: Arc<dyn meerkat_application::ports::oidc_config_read_store::OidcConfigReadStore>,
    project: Arc<dyn meerkat_application::ports::project_read_store::ProjectReadStore>,
    member: Arc<dyn meerkat_application::ports::member_read_store::MemberReadStore>,
    project_role: Arc<dyn meerkat_application::ports::project_role_read_store::ProjectRoleReadStore>,
    project_member: Arc<dyn meerkat_application::ports::project_member_read_store::ProjectMemberReadStore>,
    project_permission: Arc<dyn meerkat_application::ports::project_permission_read_store::ProjectPermissionReadStore>,
    project_key: Arc<dyn meerkat_application::ports::project_key_read_store::ProjectKeyReadStore>,
    issue: Arc<dyn meerkat_application::ports::issue_read_store::IssueReadStore>,
    event: Arc<dyn meerkat_application::ports::event_read_store::EventReadStore>,
}

struct MediatorDeps {
    audit_logger: Arc<dyn meerkat_application::ports::audit::AuditLogger>,
    project_permission_store: Arc<dyn meerkat_application::ports::project_permission_read_store::ProjectPermissionReadStore>,
    org_read_store: Arc<dyn meerkat_application::ports::organization_read_store::OrganizationReadStore>,
    oidc_config_read_store: Arc<dyn meerkat_application::ports::oidc_config_read_store::OidcConfigReadStore>,
    project_read_store: Arc<dyn meerkat_application::ports::project_read_store::ProjectReadStore>,
    member_read_store: Arc<dyn meerkat_application::ports::member_read_store::MemberReadStore>,
    project_role_read_store: Arc<dyn meerkat_application::ports::project_role_read_store::ProjectRoleReadStore>,
    project_member_read_store: Arc<dyn meerkat_application::ports::project_member_read_store::ProjectMemberReadStore>,
    project_key_read_store: Arc<dyn meerkat_application::ports::project_key_read_store::ProjectKeyReadStore>,
    issue_read_store: Arc<dyn meerkat_application::ports::issue_read_store::IssueReadStore>,
    event_read_store: Arc<dyn meerkat_application::ports::event_read_store::EventReadStore>,
    fingerprint_service: Arc<dyn meerkat_application::ports::fingerprint_service::FingerprintService>,
}

fn build_mediator(deps: MediatorDeps) -> Mediator<RequestContext, ApplicationError> {
    let mut mediator = Mediator::new();
    mediator.add_behavior(Arc::new(RateLimitBehavior::new()));
    mediator.add_behavior(Arc::new(AuthorizationBehavior::new(deps.audit_logger, deps.project_permission_store.clone(), deps.project_read_store.clone())));

    let mut event_dispatcher = EventDispatcher::new();
    event_dispatcher.register(Arc::new(GenerateProjectKeyOnProjectCreated));
    event_dispatcher.register(Arc::new(RegressResolvedIssueOnNewEvent));
    mediator.add_behavior(Arc::new(UnitOfWorkBehavior::new(Arc::new(event_dispatcher))));
    mediator.register::<CreateOrganization, _>(CreateOrganizationHandler);
    mediator.register::<RenameOrganization, _>(RenameOrganizationHandler);
    mediator.register::<DeleteOrganization, _>(DeleteOrganizationHandler);
    mediator.register::<GetOrganization, _>(GetOrganizationHandler::new(deps.org_read_store));
    mediator.register::<GetOidcConfig, _>(GetOidcConfigHandler::new(deps.oidc_config_read_store.clone()));
    mediator.register::<ListOidcConfigs, _>(ListOidcConfigsHandler::new(deps.oidc_config_read_store));
    mediator.register::<AddOidcConfig, _>(AddOidcConfigHandler);
    mediator.register::<ActivateOidcConfig, _>(ActivateOidcConfigHandler);
    mediator.register::<DeleteOidcConfig, _>(DeleteOidcConfigHandler);
    mediator.register::<UpdateOidcClaimMapping, _>(UpdateOidcClaimMappingHandler);
    mediator.register::<CreateProject, _>(CreateProjectHandler);
    mediator.register::<RenameProject, _>(RenameProjectHandler);
    mediator.register::<DeleteProject, _>(DeleteProjectHandler);
    mediator.register::<GetProject, _>(GetProjectHandler::new(deps.project_read_store.clone()));
    mediator.register::<ListProjects, _>(ListProjectsHandler::new(deps.project_read_store.clone()));
    mediator.register::<GetCurrentUser, _>(GetCurrentUserHandler::new(deps.project_permission_store.clone()));
    mediator.register::<ListMembers, _>(ListMembersHandler::new(deps.member_read_store.clone()));
    mediator.register::<GetMemberAccess, _>(GetMemberAccessHandler::new(deps.member_read_store, deps.project_member_read_store.clone()));
    mediator.register::<ListMemberProjects, _>(ListMemberProjectsHandler::new(deps.project_member_read_store.clone()));
    mediator.register::<ListProjectRoles, _>(ListProjectRolesHandler::new(deps.project_read_store.clone(), deps.project_role_read_store));
    mediator.register::<ListProjectMembers, _>(ListProjectMembersHandler::new(deps.project_read_store.clone(), deps.project_member_read_store));
    mediator.register::<ListProjectKeys, _>(ListProjectKeysHandler::new(deps.project_read_store.clone(), deps.project_key_read_store));
    mediator.register::<GetIssue, _>(GetIssueHandler::new(deps.project_read_store.clone(), deps.issue_read_store.clone()));
    mediator.register::<ListIssues, _>(ListIssuesHandler::new(deps.project_read_store.clone(), deps.issue_read_store.clone()));
    mediator.register::<ListIssueEvents, _>(ListIssueEventsHandler::new(deps.project_read_store.clone(), deps.issue_read_store, deps.event_read_store));
    mediator.register::<ResolveIssue, _>(ResolveIssueHandler::new(deps.project_read_store.clone()));
    mediator.register::<ReopenIssue, _>(ReopenIssueHandler::new(deps.project_read_store.clone()));
    mediator.register::<IgnoreIssue, _>(IgnoreIssueHandler::new(deps.project_read_store.clone()));
    mediator.register::<CreateProjectKey, _>(CreateProjectKeyHandler);
    mediator.register::<RevokeProjectKey, _>(RevokeProjectKeyHandler);
    mediator.register::<UpdateProjectKeyRateLimit, _>(UpdateProjectKeyRateLimitHandler);
    mediator.register::<IngestEvent, _>(IngestEventHandler::new(deps.fingerprint_service));
    mediator
}

fn build_read_stores(pool: &PgPool) -> ReadStores {
    ReadStores {
        org: Arc::new(PgOrganizationReadStore::new(pool.clone())),
        oidc_config: Arc::new(PgOidcConfigReadStore::new(pool.clone())),
        project: Arc::new(PgProjectReadStore::new(pool.clone())),
        member: Arc::new(meerkat_infrastructure::persistence::pg_member_read_store::PgMemberReadStore::new(pool.clone())),
        project_role: Arc::new(meerkat_infrastructure::persistence::pg_project_role_read_store::PgProjectRoleReadStore::new(pool.clone())),
        project_member: Arc::new(meerkat_infrastructure::persistence::pg_project_member_read_store::PgProjectMemberReadStore::new(pool.clone())),
        project_permission: Arc::new(PgProjectPermissionReadStore::new(pool.clone())),
        project_key: Arc::new(meerkat_infrastructure::persistence::pg_project_key_read_store::PgProjectKeyReadStore::new(pool.clone())),
        issue: Arc::new(meerkat_infrastructure::persistence::pg_issue_read_store::PgIssueReadStore::new(pool.clone())),
        event: Arc::new(meerkat_infrastructure::persistence::pg_event_read_store::PgEventReadStore::new(pool.clone())),
    }
}

fn build_app_state(pool: &PgPool, config: &MeerkatConfig, stores: &ReadStores) -> AppState {
    let uow_factory = Arc::new(PgUnitOfWorkFactory::new(pool.clone(), Arc::new(SystemClock)));
    let context = Arc::new(AppContext::new(
        uow_factory,
        Arc::new(ErrorPipeline::new(vec![Arc::new(TracingErrorObserver)])),
    ));

    let audit_logger: Arc<dyn meerkat_application::ports::audit::AuditLogger> = Arc::new(AuditPipeline::new(vec![
        Arc::new(TracingAuditLogger),
    ]));

    let fingerprint_service = Arc::new(meerkat_infrastructure::sha256_fingerprint_service::Sha256FingerprintService);

    let mediator = Arc::new(build_mediator(MediatorDeps {
        audit_logger,
        project_permission_store: stores.project_permission.clone(),
        org_read_store: stores.org.clone(),
        oidc_config_read_store: stores.oidc_config.clone(),
        project_read_store: stores.project.clone(),
        member_read_store: stores.member.clone(),
        project_role_read_store: stores.project_role.clone(),
        project_member_read_store: stores.project_member.clone(),
        project_key_read_store: stores.project_key.clone(),
        issue_read_store: stores.issue.clone(),
        event_read_store: stores.event.clone(),
        fingerprint_service,
    }));

    AppState {
        health_checker: Arc::new(PgHealthChecker::new(pool.clone())),
        mediator,
        context,
        auth: AuthState {
            oidc_config_read_store: stores.oidc_config.clone(),
            jwks_provider: Arc::new(CachedJwksProvider::new(std::time::Duration::from_secs(300))),
            oidc_discovery_provider: Arc::new(CachedOidcDiscoveryProvider::new(std::time::Duration::from_secs(300))),
            member_repository: Arc::new(PgMemberRepository::new(pool.clone())),
        },
        tenant: TenantState {
            org_read_store: stores.org.clone(),
            base_domain: config.base_domain.clone(),
            master_org_slug: config.master_org_slug.clone(),
        },
        project_key_read_store: stores.project_key.clone(),
        auth_enabled: true,
    }
}

async fn serve(
    listen_addr: &str,
    state: AppState,
    mut shutdown: watch::Receiver<bool>,
) -> anyhow::Result<()> {
    let router = meerkat_api::router(state);

    let listener = tokio::net::TcpListener::bind(listen_addr)
        .await
        .with_context(|| format!("Failed to bind to {}", listen_addr))?;

    info!("Listening on {}", listen_addr);

    axum::serve(listener, router)
        .with_graceful_shutdown(async move {
            let _ = shutdown.changed().await;
        })
        .await
        .context("Server error")
}

async fn run_api(
    pool: PgPool,
    config: &MeerkatConfig,
    shutdown: watch::Receiver<bool>,
) -> anyhow::Result<()> {
    let stores = build_read_stores(&pool);
    let state = build_app_state(&pool, config, &stores);
    serve(&config.listen_addr, state, shutdown).await
}
