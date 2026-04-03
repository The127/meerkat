use std::sync::Arc;

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::{Extension, Json};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use meerkat_application::context::RequestContext;
use meerkat_application::error::ApplicationError;
use meerkat_application::organizations::activate_oidc_config::ActivateOidcConfig;
use meerkat_application::organizations::add_oidc_config::AddOidcConfig;
use meerkat_application::organizations::delete_oidc_config::DeleteOidcConfig;
use meerkat_application::organizations::list_oidc_configs::ListOidcConfigs;
use meerkat_domain::models::oidc_config::{Audience, ClaimMapping, ClientId, OidcConfigId};
use meerkat_domain::models::organization::OrganizationIdentifier;
use meerkat_domain::shared::url::Url;

use super::vec1_from_dto;
use crate::error::ApiError;
use crate::handlers::oidc::ClaimMappingResponseDto;
use crate::handlers::organizations::ClaimMappingDto;
use crate::resolved_organization::ResolvedOrganization;
use crate::state::AppState;

// --- List ---

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct OidcConfigListItemDto {
    #[serde(rename = "id")]
    pub id: OidcConfigId,
    #[serde(rename = "name")]
    pub name: String,
    #[serde(rename = "client_id")]
    pub client_id: String,
    #[serde(rename = "issuer_url")]
    pub issuer_url: String,
    #[serde(rename = "audience")]
    pub audience: String,
    #[serde(rename = "discovery_url", skip_serializing_if = "Option::is_none")]
    pub discovery_url: Option<String>,
    #[serde(rename = "claim_mapping")]
    pub claim_mapping: ClaimMappingResponseDto,
    #[serde(rename = "status")]
    pub status: String,
}

#[utoipa::path(
    get,
    path = "/api/v1/organization/oidc-configs",
    responses(
        (status = 200, description = "List of OIDC configurations", body = Vec<OidcConfigListItemDto>),
    )
)]
pub(crate) async fn list_oidc_configs(
    State(state): State<AppState>,
    Extension(req_ctx): Extension<Arc<RequestContext>>,
    Extension(resolved_org): Extension<ResolvedOrganization>,
) -> Result<Json<Vec<OidcConfigListItemDto>>, ApiError> {
    let configs = state
        .mediator
        .dispatch(ListOidcConfigs { org_id: resolved_org.id }, &req_ctx)
        .await?;

    let items = configs
        .into_iter()
        .map(|c| {
            let cm = c.claim_mapping;
            OidcConfigListItemDto {
                id: c.id,
                name: c.name,
                client_id: c.client_id.as_str().to_string(),
                issuer_url: c.issuer_url.as_str().to_string(),
                audience: c.audience.as_str().to_string(),
                discovery_url: c.discovery_url.map(|u| u.as_str().to_string()),
                claim_mapping: ClaimMappingResponseDto {
                    sub_claim: cm.sub_claim().as_str().to_string(),
                    name_claim: cm.name_claim().as_str().to_string(),
                    role_claim: cm.role_claim().as_str().to_string(),
                    owner_values: cm.owner_values().to_vec(),
                    admin_values: cm.admin_values().to_vec(),
                    member_values: cm.member_values().to_vec(),
                },
                status: c.status.to_string(),
            }
        })
        .collect();

    Ok(Json(items))
}

// --- Add ---

#[derive(Debug, Deserialize, ToSchema)]
pub(crate) struct AddOidcConfigRequestDto {
    #[serde(rename = "name")]
    pub name: String,
    #[serde(rename = "client_id")]
    pub client_id: ClientId,
    #[serde(rename = "issuer_url")]
    pub issuer_url: Url,
    #[serde(rename = "audience")]
    pub audience: Audience,
    #[serde(rename = "discovery_url")]
    pub discovery_url: Option<Url>,
    #[serde(rename = "claim_mapping")]
    pub claim_mapping: ClaimMappingDto,
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct AddOidcConfigResponseDto {
    #[serde(rename = "id")]
    pub id: OidcConfigId,
}

#[utoipa::path(
    post,
    path = "/api/v1/organization/oidc-configs",
    request_body = AddOidcConfigRequestDto,
    responses(
        (status = 201, description = "OIDC config added", body = AddOidcConfigResponseDto),
        (status = 400, description = "Validation error"),
    )
)]
pub(crate) async fn add_oidc_config(
    State(state): State<AppState>,
    Extension(req_ctx): Extension<Arc<RequestContext>>,
    Extension(resolved_org): Extension<ResolvedOrganization>,
    Json(body): Json<AddOidcConfigRequestDto>,
) -> Result<(StatusCode, Json<AddOidcConfigResponseDto>), ApiError> {
    let cm = body.claim_mapping;
    let claim_mapping = ClaimMapping::new(
        cm.sub_claim, cm.name_claim, cm.role_claim,
        vec1_from_dto(cm.owner_values, "owner_values")?,
        vec1_from_dto(cm.admin_values, "admin_values")?,
        vec1_from_dto(cm.member_values, "member_values")?,
    ).map_err(|e| ApplicationError::Validation(e.to_string()))?;

    let cmd = AddOidcConfig {
        identifier: OrganizationIdentifier::Id(resolved_org.id),
        name: body.name,
        client_id: body.client_id,
        issuer_url: body.issuer_url,
        audience: body.audience,
        discovery_url: body.discovery_url,
        claim_mapping,
    };

    let id = state.mediator.dispatch(cmd, &req_ctx).await?;

    Ok((StatusCode::CREATED, Json(AddOidcConfigResponseDto { id })))
}

// --- Activate ---

#[utoipa::path(
    post,
    path = "/api/v1/organization/oidc-configs/{id}/activate",
    responses(
        (status = 204, description = "OIDC config activated"),
        (status = 404, description = "Config not found"),
        (status = 400, description = "Invalid status transition"),
    )
)]
pub(crate) async fn activate_oidc_config(
    State(state): State<AppState>,
    Extension(req_ctx): Extension<Arc<RequestContext>>,
    Extension(resolved_org): Extension<ResolvedOrganization>,
    Path(config_id): Path<OidcConfigId>,
) -> Result<StatusCode, ApiError> {
    let cmd = ActivateOidcConfig {
        org_identifier: OrganizationIdentifier::Id(resolved_org.id),
        config_id,
    };

    state.mediator.dispatch(cmd, &req_ctx).await?;

    Ok(StatusCode::NO_CONTENT)
}

// --- Delete ---

#[utoipa::path(
    delete,
    path = "/api/v1/organization/oidc-configs/{id}",
    responses(
        (status = 204, description = "OIDC config deleted"),
        (status = 404, description = "Config not found"),
        (status = 400, description = "Cannot delete active config"),
    )
)]
pub(crate) async fn delete_oidc_config(
    State(state): State<AppState>,
    Extension(req_ctx): Extension<Arc<RequestContext>>,
    Extension(resolved_org): Extension<ResolvedOrganization>,
    Path(config_id): Path<OidcConfigId>,
) -> Result<StatusCode, ApiError> {
    let cmd = DeleteOidcConfig {
        org_identifier: OrganizationIdentifier::Id(resolved_org.id),
        config_id,
    };

    state.mediator.dispatch(cmd, &req_ctx).await?;

    Ok(StatusCode::NO_CONTENT)
}
