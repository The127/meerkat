use std::collections::HashMap;
use std::sync::Arc;

use axum::extract::State;
use axum::{Extension, Json};
use serde::Serialize;
use utoipa::ToSchema;

use meerkat_application::context::RequestContext;
use meerkat_application::members::get_current_user::GetCurrentUser;
use meerkat_domain::models::member::MemberId;

use crate::error::ApiError;
use crate::state::AppState;

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct CurrentUserDto {
    #[serde(rename = "member_id")]
    pub member_id: MemberId,
    #[serde(rename = "preferred_name")]
    pub preferred_name: String,
    #[serde(rename = "org_permissions")]
    pub org_permissions: Vec<String>,
    #[serde(rename = "project_permissions")]
    pub project_permissions: HashMap<String, Vec<String>>,
}

#[utoipa::path(
    get,
    path = "/api/v1/me",
    responses(
        (status = 200, description = "Current user identity and permissions", body = CurrentUserDto),
    )
)]
pub(crate) async fn get_current_user(
    State(state): State<AppState>,
    Extension(req_ctx): Extension<Arc<RequestContext>>,
) -> Result<Json<CurrentUserDto>, ApiError> {
    let user = state.mediator.dispatch(GetCurrentUser, &req_ctx).await?;

    Ok(Json(CurrentUserDto {
        member_id: user.member_id,
        preferred_name: user.preferred_name,
        org_permissions: user.org_permissions.into_iter().map(|p| p.to_string()).collect(),
        project_permissions: user
            .project_permissions
            .into_iter()
            .map(|(slug, perms)| {
                (slug.to_string(), perms.into_iter().map(|p| p.to_string()).collect())
            })
            .collect(),
    }))
}
