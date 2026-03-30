use axum::Router;
use axum::routing::get;
use utoipa::OpenApi;
use crate::handlers::health;
use crate::state::AppState;

pub mod state;
pub mod error;
pub mod handlers;

#[derive(OpenApi)]
#[openapi(
    paths(
        health::liveness,
    ),
    components(schemas(
        health::HealthResponse,
    ))
)]
struct ApiDoc;

pub fn router(state: AppState) -> Router {
    let app_routes = Router::new();

    let api_routes = Router::new()
        .nest("/api/v1/applications", app_routes);

    Router::new()
        .merge(api_routes)
        .route("/api/openapi.json", get(|| async { axum::Json(ApiDoc::openapi()) }))
        .route("/health", get(health::liveness))
        .with_state(state)
}
