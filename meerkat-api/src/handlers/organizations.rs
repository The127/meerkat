use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use meerkat_application::context::RequestContext;
use meerkat_application::organizations::create::CreateOrganization;
use meerkat_domain::models::organization::{OrganizationId, OrganizationSlug};

use crate::error::ApiError;
use crate::state::AppState;

#[derive(Debug, Deserialize, ToSchema)]
pub(crate) struct CreateOrganizationRequestDto {
    #[serde(rename = "name")]
    pub name: String,
    #[serde(rename = "slug")]
    #[schema(value_type = String)]
    pub slug: OrganizationSlug,
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct CreateOrganizationResponseDto {
    #[serde(rename = "id")]
    #[schema(value_type = uuid::Uuid)]
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
    Json(body): Json<CreateOrganizationRequestDto>,
) -> Result<(StatusCode, Json<CreateOrganizationResponseDto>), ApiError> {
    let cmd = CreateOrganization {
        name: body.name,
        slug: body.slug,
    };

    let req_ctx = RequestContext::new(state.context.clone());
    let id = state.mediator.dispatch(cmd, &req_ctx).await?;

    Ok((StatusCode::CREATED, Json(CreateOrganizationResponseDto { id })))
}
