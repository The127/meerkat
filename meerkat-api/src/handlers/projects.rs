use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use meerkat_application::context::RequestContext;
use meerkat_application::projects::create::CreateProject;
use meerkat_domain::models::organization::OrganizationId;
use meerkat_domain::models::project::{ProjectId, ProjectSlug};

use crate::error::ApiError;
use crate::state::AppState;

#[derive(Debug, Deserialize, ToSchema)]
pub(crate) struct CreateProjectRequestDto {
    #[serde(rename = "organization_id")]
    #[schema(value_type = uuid::Uuid)]
    pub organization_id: OrganizationId,
    #[serde(rename = "name")]
    pub name: String,
    #[serde(rename = "slug")]
    #[schema(value_type = String)]
    pub slug: ProjectSlug,
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct CreateProjectResponseDto {
    #[serde(rename = "id")]
    #[schema(value_type = uuid::Uuid)]
    pub id: ProjectId,
}

#[utoipa::path(
    post,
    path = "/api/v1/projects",
    request_body = CreateProjectRequestDto,
    responses(
        (status = 201, description = "Project created", body = CreateProjectResponseDto),
        (status = 400, description = "Validation error"),
        (status = 500, description = "Internal server error"),
    )
)]
pub(crate) async fn create_project(
    State(state): State<AppState>,
    Json(body): Json<CreateProjectRequestDto>,
) -> Result<(StatusCode, Json<CreateProjectResponseDto>), ApiError> {
    let cmd = CreateProject {
        organization_id: body.organization_id,
        name: body.name,
        slug: body.slug,
    };

    let req_ctx = RequestContext::new(state.context.clone());
    let id = state.mediator.dispatch(cmd, &req_ctx).await?;

    Ok((StatusCode::CREATED, Json(CreateProjectResponseDto { id })))
}
