pub mod config;

use std::sync::Arc;
use anyhow::Context;
use clap::{Parser, Subcommand};
use sqlx::PgPool;
use tokio::sync::watch;
use tracing::info;
use meerkat_api::state::AppState;
use meerkat_application::context::AppContext;
use meerkat_application::error::ApplicationError;
use meerkat_application::mediator::Mediator;
use meerkat_application::organizations::create::{CreateOrganization, CreateOrganizationHandler};
use meerkat_application::ports::error_observer::ErrorPipeline;
use meerkat_infrastructure::clock::SystemClock;
use meerkat_infrastructure::persistence::pg_unit_of_work::PgUnitOfWorkFactory;
use meerkat_infrastructure::persistence::pq_health_checker::PgHealthChecker;
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

            let (shutdown_tx, shutdown_rx) = watch::channel(false);

            let api_handle = tokio::spawn(async move {
                run_api(pool, &config.listen_addr, shutdown_rx).await
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

fn build_mediator() -> Mediator<AppContext, ApplicationError> {
    let mut mediator = Mediator::new();
    mediator.register::<CreateOrganization, _>(CreateOrganizationHandler);
    mediator
}

async fn run_api(
    pool: PgPool,
    listen_addr: &str,
    mut shutdown: watch::Receiver<bool>,
) -> anyhow::Result<()> {
    let health_checker = Arc::new(PgHealthChecker::new(pool.clone()));

    let uow_factory = Arc::new(PgUnitOfWorkFactory::new(pool.clone()));

    let error_observer = Arc::new(ErrorPipeline::new(vec![
        Arc::new(TracingErrorObserver),
    ]));

    let context = Arc::new(AppContext {
        clock: Arc::new(SystemClock),
        uow_factory,
        error_observer,
    });

    let mediator = Arc::new(build_mediator());

    let state = AppState {
        health_checker,
        mediator,
        context,
    };

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
        .context("Server error")?;

    Ok(())
}
