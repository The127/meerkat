use std::sync::Arc;

use axum::extract::State;
use axum::http::StatusCode;
use axum::{Extension, Json};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use meerkat_application::context::RequestContext;
use meerkat_application::error::ApplicationError;
use meerkat_application::organizations::create::{CreateOrganization, CreateOrganizationOidcConfig};
use meerkat_application::organizations::delete::DeleteOrganization;
use meerkat_application::organizations::get::GetOrganization;
use meerkat_application::organizations::rename::RenameOrganization;
use meerkat_domain::models::oidc_config::{Audience, ClaimMapping, ClientId};
use meerkat_domain::shared::url::Url;

use super::role_values_from_dto;
use meerkat_domain::models::organization::{OrganizationId, OrganizationIdentifier, OrganizationSlug};

use crate::error::ApiError;
use crate::resolved_organization::ResolvedOrganization;
use crate::state::AppState;

#[derive(Debug, Deserialize, ToSchema)]
pub(crate) struct RoleValuesDto {
    #[serde(rename = "owner")]
    pub owner: Vec<String>,
    #[serde(rename = "admin")]
    pub admin: Vec<String>,
    #[serde(rename = "member")]
    pub member: Vec<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub(crate) struct ClaimMappingDto {
    #[serde(rename = "sub_claim")]
    pub sub_claim: String,
    #[serde(rename = "name_claim")]
    pub name_claim: String,
    #[serde(rename = "role_claim")]
    pub role_claim: String,
    #[serde(rename = "role_values")]
    pub role_values: RoleValuesDto,
}

impl TryFrom<ClaimMappingDto> for ClaimMapping {
    type Error = ApplicationError;

    fn try_from(dto: ClaimMappingDto) -> Result<Self, Self::Error> {
        let role_values = role_values_from_dto(dto.role_values)?;
        ClaimMapping::new(dto.sub_claim, dto.name_claim, dto.role_claim, role_values)
            .map_err(|e| ApplicationError::Validation(e.to_string()))
    }
}

#[derive(Debug, Deserialize, ToSchema)]
pub(crate) struct CreateOrganizationOidcConfigDto {
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

#[derive(Debug, Deserialize, ToSchema)]
pub(crate) struct CreateOrganizationRequestDto {
    #[serde(rename = "name")]
    pub name: String,
    #[serde(rename = "slug")]
    pub slug: OrganizationSlug,
    #[serde(rename = "oidc_config")]
    pub oidc_config: CreateOrganizationOidcConfigDto,
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct CreateOrganizationResponseDto {
    #[serde(rename = "id")]
    pub id: OrganizationId,
}

#[utoipa::path(
    post,
    path = "/api/v1/organizations",
    request_body = CreateOrganizationRequestDto,
    responses(
        (status = 201, description = "Organization created", body = CreateOrganizationResponseDto),
        (status = 400, description = "Validation error"),
        (status = 500, description = "Internal server error"),
    )
)]
pub(crate) async fn create_organization(
    State(state): State<AppState>,
    Extension(req_ctx): Extension<Arc<RequestContext>>,
    Json(body): Json<CreateOrganizationRequestDto>,
) -> Result<(StatusCode, Json<CreateOrganizationResponseDto>), ApiError> {
    let oidc = body.oidc_config;

    let claim_mapping = ClaimMapping::try_from(oidc.claim_mapping)?;

    let cmd = CreateOrganization {
        name: body.name,
        slug: body.slug,
        oidc_config: CreateOrganizationOidcConfig {
            name: oidc.name,
            client_id: oidc.client_id,
            issuer_url: oidc.issuer_url,
            audience: oidc.audience,
            discovery_url: oidc.discovery_url,
            claim_mapping,
        },
    };

    let id = state.mediator.dispatch(cmd, &req_ctx).await?;

    Ok((StatusCode::CREATED, Json(CreateOrganizationResponseDto { id })))
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct OrganizationDto {
    #[serde(rename = "id")]
    pub id: OrganizationId,
    #[serde(rename = "slug")]
    pub slug: OrganizationSlug,
    #[serde(rename = "name")]
    pub name: String,
}

#[utoipa::path(
    get,
    path = "/api/v1/organization",
    responses(
        (status = 200, description = "Current organization", body = OrganizationDto),
    )
)]
pub(crate) async fn get_organization(
    State(state): State<AppState>,
    Extension(req_ctx): Extension<Arc<RequestContext>>,
    Extension(resolved_org): Extension<ResolvedOrganization>,
) -> Result<Json<OrganizationDto>, ApiError> {
    let query = GetOrganization {
        identifier: resolved_org.id.into(),
    };

    let org = state.mediator.dispatch(query, &req_ctx).await?;

    Ok(Json(OrganizationDto {
        id: org.id,
        slug: org.slug,
        name: org.name,
    }))
}

#[derive(Debug, Deserialize, ToSchema)]
pub(crate) struct RenameOrganizationRequestDto {
    #[serde(rename = "name")]
    pub name: String,
}

#[utoipa::path(
    post,
    path = "/api/v1/organization/rename",
    request_body = RenameOrganizationRequestDto,
    responses(
        (status = 204, description = "Organization renamed"),
        (status = 400, description = "Validation error"),
        (status = 404, description = "Organization not found"),
        (status = 409, description = "Conflict"),
    )
)]
pub(crate) async fn rename_organization(
    State(state): State<AppState>,
    Extension(req_ctx): Extension<Arc<RequestContext>>,
    Extension(resolved_org): Extension<ResolvedOrganization>,
    Json(body): Json<RenameOrganizationRequestDto>,
) -> Result<StatusCode, ApiError> {
    let cmd = RenameOrganization {
        identifier: OrganizationIdentifier::Id(resolved_org.id),
        name: body.name,
    };

    state.mediator.dispatch(cmd, &req_ctx).await?;

    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    delete,
    path = "/api/v1/organization",
    responses(
        (status = 204, description = "Organization deleted"),
        (status = 404, description = "Organization not found"),
    )
)]
pub(crate) async fn delete_organization(
    State(state): State<AppState>,
    Extension(req_ctx): Extension<Arc<RequestContext>>,
    Extension(resolved_org): Extension<ResolvedOrganization>,
) -> Result<StatusCode, ApiError> {
    let cmd = DeleteOrganization {
        identifier: OrganizationIdentifier::Id(resolved_org.id),
    };

    state.mediator.dispatch(cmd, &req_ctx).await?;

    Ok(StatusCode::NO_CONTENT)
}
