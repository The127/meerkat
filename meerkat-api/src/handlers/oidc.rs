use std::sync::Arc;

use axum::extract::State;
use axum::{Extension, Json};
use serde::Serialize;
use utoipa::ToSchema;

use meerkat_application::context::RequestContext;
use meerkat_application::organizations::get_oidc_config::GetOidcConfig;
use meerkat_domain::models::oidc_config::ClaimMapping;

use crate::error::ApiError;
use crate::resolved_organization::ResolvedOrganization;
use crate::state::AppState;

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct RoleValuesResponseDto {
    #[serde(rename = "owner")]
    pub owner: Vec<String>,
    #[serde(rename = "admin")]
    pub admin: Vec<String>,
    #[serde(rename = "member")]
    pub member: Vec<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct ClaimMappingResponseDto {
    #[serde(rename = "sub_claim")]
    pub sub_claim: String,
    #[serde(rename = "name_claim")]
    pub name_claim: String,
    #[serde(rename = "role_claim")]
    pub role_claim: String,
    #[serde(rename = "role_values")]
    pub role_values: RoleValuesResponseDto,
}

impl From<&ClaimMapping> for ClaimMappingResponseDto {
    fn from(cm: &ClaimMapping) -> Self {
        Self {
            sub_claim: cm.sub_claim().as_str().to_string(),
            name_claim: cm.name_claim().as_str().to_string(),
            role_claim: cm.role_claim().as_str().to_string(),
            role_values: RoleValuesResponseDto {
                owner: cm.role_values().owner().to_vec(),
                admin: cm.role_values().admin().to_vec(),
                member: cm.role_values().member().to_vec(),
            },
        }
    }
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
    Extension(req_ctx): Extension<Arc<RequestContext>>,
    Extension(resolved_org): Extension<ResolvedOrganization>,
) -> Result<Json<OidcConfigDto>, ApiError> {
    let query = GetOidcConfig {
        org_id: resolved_org.id,
    };

    let config = state.mediator.dispatch(query, &req_ctx).await?;

    Ok(Json(OidcConfigDto {
        name: config.name,
        client_id: config.client_id.as_str().to_string(),
        issuer_url: config.issuer_url.as_str().to_string(),
        audience: config.audience.as_str().to_string(),
        discovery_url: config.discovery_url.map(|u| u.as_str().to_string()),
        claim_mapping: ClaimMappingResponseDto::from(&config.claim_mapping),
    }))
}
