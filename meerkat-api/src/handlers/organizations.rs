use std::sync::Arc;

use axum::extract::State;
use axum::http::StatusCode;
use axum::{Extension, Json};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use meerkat_application::context::RequestContext;
use meerkat_application::organizations::create::{CreateOrganization, CreateOrganizationOidcConfig};
use meerkat_domain::models::oidc_config::{Audience, ClientId};
use meerkat_domain::shared::url::Url;
use meerkat_domain::models::organization::{OrganizationId, OrganizationSlug};

use crate::error::ApiError;
use crate::resolved_organization::ResolvedOrganization;
use crate::state::AppState;

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

    let cmd = CreateOrganization {
        name: body.name,
        slug: body.slug,
        oidc_config: CreateOrganizationOidcConfig {
            name: oidc.name,
            client_id: oidc.client_id,
            issuer_url: oidc.issuer_url,
            audience: oidc.audience,
            discovery_url: oidc.discovery_url,
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
    Extension(resolved_org): Extension<ResolvedOrganization>,
) -> Json<OrganizationDto> {
    Json(OrganizationDto {
        id: resolved_org.id,
        slug: resolved_org.slug,
        name: resolved_org.name,
    })
}
