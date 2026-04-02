use axum::extract::State;
use axum::{Extension, Json};
use serde::Serialize;
use utoipa::ToSchema;

use crate::error::ApiError;
use crate::resolved_organization::ResolvedOrganization;
use crate::state::AppState;

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct ClaimMappingResponseDto {
    #[serde(rename = "sub_claim")]
    pub sub_claim: String,
    #[serde(rename = "name_claim")]
    pub name_claim: String,
    #[serde(rename = "role_claim")]
    pub role_claim: String,
    #[serde(rename = "owner_values")]
    pub owner_values: Vec<String>,
    #[serde(rename = "admin_values")]
    pub admin_values: Vec<String>,
    #[serde(rename = "member_values")]
    pub member_values: Vec<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct OidcConfigDto {
    pub name: String,
    pub client_id: String,
    pub issuer_url: String,
    pub audience: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub discovery_url: Option<String>,
    pub claim_mapping: ClaimMappingResponseDto,
}

#[utoipa::path(
    get,
    path = "/api/v1/oidc",
    responses(
        (status = 200, description = "Active OIDC configuration for the resolved organization", body = OidcConfigDto),
        (status = 404, description = "No active OIDC configuration found"),
    ),
)]
pub(crate) async fn get_oidc_config(
    State(state): State<AppState>,
    Extension(resolved_org): Extension<ResolvedOrganization>,
) -> Result<Json<OidcConfigDto>, ApiError> {
    let config = state
        .oidc_config_read_store
        .find_active_by_org_id(&resolved_org.id)
        .await?;

    let cm = config.claim_mapping;
    Ok(Json(OidcConfigDto {
        name: config.name,
        client_id: config.client_id.as_str().to_string(),
        issuer_url: config.issuer_url.as_str().to_string(),
        audience: config.audience.as_str().to_string(),
        discovery_url: config.discovery_url.map(|u| u.as_str().to_string()),
        claim_mapping: ClaimMappingResponseDto {
            sub_claim: cm.sub_claim().as_str().to_string(),
            name_claim: cm.name_claim().as_str().to_string(),
            role_claim: cm.role_claim().as_str().to_string(),
            owner_values: cm.owner_values().to_vec(),
            admin_values: cm.admin_values().to_vec(),
            member_values: cm.member_values().to_vec(),
        },
    }))
}
