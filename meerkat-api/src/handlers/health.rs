use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use axum::response::IntoResponse;
use serde::Serialize;
use utoipa::ToSchema;
use crate::state::AppState;

#[derive(Serialize, ToSchema)]
pub(crate) struct HealthDto {
    #[serde(rename = "status")]
    pub status: String,
}

#[utoipa::path(
    get,
    path = "/health",
    responses(
        (status = 200, description = "Health check response", body = HealthDto)
    )
)]
pub(crate) async fn liveness() -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(HealthDto {
            status: "healthy".to_string(),
        })
    )
}

#[utoipa::path(
    get,
    path = "/health/ready",
    responses(
        (status = 200, description = "Readiness check response", body = HealthDto),
        (status = 503, description = "Service unavailable", body = HealthDto)
    )
)]
pub(crate) async fn readiness(State(state): State<AppState>) -> impl IntoResponse {
    if state.health_checker.check().await {
        (
            StatusCode::OK,
            Json(HealthDto {
                status: "healthy".to_string(),
            })
        )
    } else {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(HealthDto {
                status: "not ready".to_string(),
            })
        )
    }
}
