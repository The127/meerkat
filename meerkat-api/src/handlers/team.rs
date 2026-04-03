use std::sync::Arc;

use axum::extract::State;
use axum::{Extension, Json};
use chrono::{DateTime, Utc};
use serde::Serialize;
use utoipa::ToSchema;

use meerkat_application::context::RequestContext;
use meerkat_application::members::list_members::ListMembers;
use meerkat_domain::models::member::MemberId;

use crate::error::ApiError;
use crate::resolved_organization::ResolvedOrganization;
use crate::state::AppState;

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct MemberDto {
    #[serde(rename = "id")]
    pub id: MemberId,
    #[serde(rename = "sub")]
    pub sub: String,
    #[serde(rename = "preferred_name")]
    pub preferred_name: String,
    #[serde(rename = "org_roles")]
    pub org_roles: Vec<String>,
    #[serde(rename = "created_at")]
    pub created_at: DateTime<Utc>,
}

#[utoipa::path(
    get,
    path = "/api/v1/members",
    responses(
        (status = 200, description = "List of organization members", body = Vec<MemberDto>),
    )
)]
pub(crate) async fn list_members(
    State(state): State<AppState>,
    Extension(req_ctx): Extension<Arc<RequestContext>>,
    Extension(resolved_org): Extension<ResolvedOrganization>,
) -> Result<Json<Vec<MemberDto>>, ApiError> {
    let members = state
        .mediator
        .dispatch(ListMembers { org_id: resolved_org.id }, &req_ctx)
        .await?;

    let items = members
        .into_iter()
        .map(|m| MemberDto {
            id: m.id,
            sub: m.sub,
            preferred_name: m.preferred_name,
            org_roles: m.org_roles.into_iter().map(|r| r.to_string()).collect(),
            created_at: m.created_at,
        })
        .collect();

    Ok(Json(items))
}
