use axum::extract::State;
use axum::http::StatusCode;
use axum::{Extension, Json};
use serde::Serialize;
use utoipa::ToSchema;

use crate::error::ErrorDto;
use crate::resolved_organization::ResolvedOrganization;
use crate::state::AppState;

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct OidcConfigDto {
    pub name: String,
    pub client_id: String,
    pub issuer_url: String,
    pub audience: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub discovery_url: Option<String>,
}

#[utoipa::path(
    get,
    path = "/api/v1/oidc",
    responses(
        (status = 200, description = "Active OIDC configuration for the resolved organization", body = OidcConfigDto),
        (status = 404, description = "No active OIDC configuration found", body = ErrorDto),
    ),
)]
pub(crate) async fn get_oidc_config(
    State(state): State<AppState>,
    Extension(resolved_org): Extension<ResolvedOrganization>,
) -> Result<Json<OidcConfigDto>, (StatusCode, Json<ErrorDto>)> {
    let config = state
        .oidc_config_read_store
        .find_active_by_org_id(&resolved_org.id)
        .await
        .map_err(|_| {
            (
                StatusCode::NOT_FOUND,
                Json(ErrorDto {
                    code: "oidc_config_not_found".to_string(),
                    message: "no active OIDC configuration found".to_string(),
                }),
            )
        })?;

    Ok(Json(OidcConfigDto {
        name: config.name,
        client_id: config.client_id.as_str().to_string(),
        issuer_url: config.issuer_url.as_str().to_string(),
        audience: config.audience.as_str().to_string(),
        discovery_url: config.discovery_url.map(|u| u.as_str().to_string()),
    }))
}
