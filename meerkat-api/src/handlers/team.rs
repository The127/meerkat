use std::str::FromStr;
use std::sync::Arc;

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::{Extension, Json};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use vec1::Vec1;

use meerkat_application::context::RequestContext;
use meerkat_application::error::ApplicationError;
use meerkat_application::members::get_member_access::{GetMemberAccess, MemberAccessResult};
use meerkat_application::members::list_member_projects::ListMemberProjects;
use meerkat_application::members::list_members::ListMembers;
use meerkat_application::ports::member_read_store::{ListMembersQuery, MemberReadModel};
use meerkat_application::ports::project_member_read_store::ProjectMemberRoleReadModel;
use meerkat_application::projects::assign_role_to_member::AssignRoleToProjectMember;
use meerkat_application::projects::create_role::CreateProjectRole;
use meerkat_application::projects::delete_role::DeleteProjectRole;
use meerkat_application::projects::list_members::ListProjectMembers;
use meerkat_application::projects::list_roles::ListProjectRoles;
use meerkat_application::projects::remove_role_from_member::RemoveRoleFromProjectMember;
use meerkat_application::projects::update_role::UpdateProjectRole;
use meerkat_application::search::SearchFilter;
use meerkat_domain::models::member::MemberId;
use meerkat_domain::models::org_role::OrgRole;
use meerkat_domain::models::permission::ProjectPermission;
use meerkat_domain::models::project::{ProjectIdentifier, ProjectSlug};
use meerkat_domain::models::project_role::ProjectRoleId;

use crate::error::ApiError;
use crate::pagination::PaginationQueryDto;
use crate::resolved_organization::ResolvedOrganization;
use crate::search::SearchQueryDto;
use crate::state::AppState;

#[derive(Debug, Deserialize, IntoParams, ToSchema)]
pub(crate) struct MemberFilterQueryDto {
    #[serde(rename = "role")]
    pub role: Option<String>,
    #[serde(rename = "project_slug")]
    pub project_slug: Option<String>,
}

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
    #[serde(rename = "last_seen")]
    pub last_seen: DateTime<Utc>,
}

impl From<MemberReadModel> for MemberDto {
    fn from(m: MemberReadModel) -> Self {
        Self {
            id: m.id,
            sub: m.sub,
            preferred_name: m.preferred_name,
            org_roles: m.org_roles.into_iter().map(|r| r.to_string()).collect(),
            created_at: m.created_at,
            last_seen: m.last_seen,
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct ListMembersResponseDto {
    #[serde(rename = "items")]
    pub items: Vec<MemberDto>,
    #[serde(rename = "total")]
    pub total: i64,
}

#[utoipa::path(
    get,
    path = "/api/v1/members",
    params(PaginationQueryDto, SearchQueryDto, MemberFilterQueryDto),
    responses(
        (status = 200, description = "List of organization members", body = ListMembersResponseDto),
    )
)]
pub(crate) async fn list_members(
    State(state): State<AppState>,
    Extension(req_ctx): Extension<Arc<RequestContext>>,
    Extension(resolved_org): Extension<ResolvedOrganization>,
    Query(pagination): Query<PaginationQueryDto>,
    Query(search): Query<SearchQueryDto>,
    Query(filter): Query<MemberFilterQueryDto>,
) -> Result<Json<ListMembersResponseDto>, ApiError> {
    let role = match filter.role.as_deref() {
        None => None,
        Some(raw) => Some(
            OrgRole::from_str(raw)
                .map_err(|_| ApplicationError::Validation(
                    format!("invalid role filter: '{raw}'"),
                ))?,
        ),
    };

    let project_slug = match filter.project_slug.as_deref() {
        None => None,
        Some(raw) => Some(
            ProjectSlug::new(raw)
                .map_err(|e| ApplicationError::Validation(
                    format!("invalid project_slug filter: {e}"),
                ))?,
        ),
    };

    let cmd = ListMembers {
        query: ListMembersQuery {
            org_id: resolved_org.id,
            search: search.search.as_deref().and_then(SearchFilter::new),
            role,
            project_slug,
            limit: pagination.limit(),
            offset: pagination.offset(),
        },
    };

    let result = state.mediator.dispatch(cmd, &req_ctx).await?;

    let items = result.items.into_iter().map(MemberDto::from).collect();

    Ok(Json(ListMembersResponseDto {
        items,
        total: result.total,
    }))
}

// --- Project roles ---

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct ProjectRoleDto {
    #[serde(rename = "id")]
    pub id: ProjectRoleId,
    #[serde(rename = "name")]
    pub name: String,
    #[serde(rename = "slug")]
    pub slug: String,
    #[serde(rename = "permissions")]
    pub permissions: Vec<String>,
    #[serde(rename = "is_default")]
    pub is_default: bool,
}

#[utoipa::path(
    get,
    path = "/api/v1/projects/{slug}/roles",
    responses(
        (status = 200, description = "List of project roles", body = Vec<ProjectRoleDto>),
        (status = 404, description = "Project not found"),
    )
)]
pub(crate) async fn list_project_roles(
    State(state): State<AppState>,
    Extension(req_ctx): Extension<Arc<RequestContext>>,
    Extension(resolved_org): Extension<ResolvedOrganization>,
    Path(slug): Path<ProjectSlug>,
) -> Result<Json<Vec<ProjectRoleDto>>, ApiError> {
    let roles = state
        .mediator
        .dispatch(ListProjectRoles { project: ProjectIdentifier::Slug(resolved_org.id, slug) }, &req_ctx)
        .await?;

    let items = roles
        .into_iter()
        .map(|r| ProjectRoleDto {
            id: r.id,
            name: r.name,
            slug: r.slug.as_str().to_string(),
            permissions: r.permissions.into_iter().map(|p| p.to_string()).collect(),
            is_default: r.is_default,
        })
        .collect();

    Ok(Json(items))
}

// --- Project members ---

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct ProjectMemberRoleDto {
    #[serde(rename = "role_id")]
    pub role_id: ProjectRoleId,
    #[serde(rename = "role_name")]
    pub role_name: String,
}

impl From<ProjectMemberRoleReadModel> for ProjectMemberRoleDto {
    fn from(r: ProjectMemberRoleReadModel) -> Self {
        Self { role_id: r.role_id, role_name: r.role_name }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct ProjectMemberDto {
    #[serde(rename = "member_id")]
    pub member_id: MemberId,
    #[serde(rename = "preferred_name")]
    pub preferred_name: String,
    #[serde(rename = "sub")]
    pub sub: String,
    #[serde(rename = "roles")]
    pub roles: Vec<ProjectMemberRoleDto>,
    #[serde(rename = "created_at")]
    pub created_at: DateTime<Utc>,
}

#[utoipa::path(
    get,
    path = "/api/v1/projects/{slug}/members",
    responses(
        (status = 200, description = "List of project members", body = Vec<ProjectMemberDto>),
        (status = 404, description = "Project not found"),
    )
)]
pub(crate) async fn list_project_members(
    State(state): State<AppState>,
    Extension(req_ctx): Extension<Arc<RequestContext>>,
    Extension(resolved_org): Extension<ResolvedOrganization>,
    Path(slug): Path<ProjectSlug>,
) -> Result<Json<Vec<ProjectMemberDto>>, ApiError> {
    let members = state
        .mediator
        .dispatch(ListProjectMembers { project: ProjectIdentifier::Slug(resolved_org.id, slug) }, &req_ctx)
        .await?;

    let items = members
        .into_iter()
        .map(|m| ProjectMemberDto {
            member_id: m.member_id,
            preferred_name: m.preferred_name,
            sub: m.sub,
            roles: m.roles.into_iter().map(ProjectMemberRoleDto::from).collect(),
            created_at: m.created_at,
        })
        .collect();

    Ok(Json(items))
}

// --- Project role mutation ---

#[derive(Debug, Deserialize, ToSchema)]
pub(crate) struct ProjectRoleRequestDto {
    #[serde(rename = "name")]
    pub name: String,
    #[serde(rename = "permissions")]
    pub permissions: Vec<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub(crate) struct AssignRoleRequestDto {
    #[serde(rename = "role_id")]
    pub role_id: ProjectRoleId,
}

fn parse_permissions(raw: Vec<String>) -> Result<Vec1<ProjectPermission>, ApplicationError> {
    let parsed: Vec<ProjectPermission> = raw
        .into_iter()
        .map(|s| {
            s.parse::<ProjectPermission>()
                .map_err(|_| ApplicationError::Validation(format!("invalid permission: '{s}'")))
        })
        .collect::<Result<_, _>>()?;
    Vec1::try_from_vec(parsed)
        .map_err(|_| ApplicationError::Validation("at least one permission is required".into()))
}

#[utoipa::path(
    post,
    path = "/api/v1/projects/{slug}/roles",
    responses(
        (status = 201, description = "Role created", body = ProjectRoleDto),
        (status = 400, description = "Validation error"),
    )
)]
pub(crate) async fn create_project_role(
    State(state): State<AppState>,
    Extension(req_ctx): Extension<Arc<RequestContext>>,
    Extension(resolved_org): Extension<ResolvedOrganization>,
    Path(slug): Path<ProjectSlug>,
    Json(body): Json<ProjectRoleRequestDto>,
) -> Result<(StatusCode, Json<ProjectRoleDto>), ApiError> {
    let permissions = parse_permissions(body.permissions)?;
    let role = state
        .mediator
        .dispatch(
            CreateProjectRole {
                project: ProjectIdentifier::Slug(resolved_org.id, slug),
                name: body.name,
                permissions,
            },
            &req_ctx,
        )
        .await?;

    let dto = ProjectRoleDto {
        id: role.id().clone(),
        name: role.name().to_string(),
        slug: role.slug().as_str().to_string(),
        permissions: role.permissions().iter().map(|p| p.to_string()).collect(),
        is_default: role.is_default(),
    };

    Ok((StatusCode::CREATED, Json(dto)))
}

#[utoipa::path(
    put,
    path = "/api/v1/projects/{slug}/roles/{role_id}",
    responses(
        (status = 204, description = "Role updated"),
        (status = 400, description = "Validation error"),
        (status = 404, description = "Role not found"),
    )
)]
pub(crate) async fn update_project_role(
    State(state): State<AppState>,
    Extension(req_ctx): Extension<Arc<RequestContext>>,
    Extension(resolved_org): Extension<ResolvedOrganization>,
    Path((slug, role_id)): Path<(ProjectSlug, ProjectRoleId)>,
    Json(body): Json<ProjectRoleRequestDto>,
) -> Result<StatusCode, ApiError> {
    let permissions = parse_permissions(body.permissions)?;
    state
        .mediator
        .dispatch(
            UpdateProjectRole {
                project: ProjectIdentifier::Slug(resolved_org.id, slug),
                role_id,
                name: body.name,
                permissions,
            },
            &req_ctx,
        )
        .await?;

    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    delete,
    path = "/api/v1/projects/{slug}/roles/{role_id}",
    responses(
        (status = 204, description = "Role deleted"),
        (status = 404, description = "Role not found"),
    )
)]
pub(crate) async fn delete_project_role(
    State(state): State<AppState>,
    Extension(req_ctx): Extension<Arc<RequestContext>>,
    Extension(resolved_org): Extension<ResolvedOrganization>,
    Path((slug, role_id)): Path<(ProjectSlug, ProjectRoleId)>,
) -> Result<StatusCode, ApiError> {
    state
        .mediator
        .dispatch(
            DeleteProjectRole {
                project: ProjectIdentifier::Slug(resolved_org.id, slug),
                role_id,
            },
            &req_ctx,
        )
        .await?;

    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    post,
    path = "/api/v1/projects/{slug}/members/{member_id}/roles",
    responses(
        (status = 204, description = "Role assigned"),
        (status = 404, description = "Member or role not found"),
    )
)]
pub(crate) async fn assign_role_to_member(
    State(state): State<AppState>,
    Extension(req_ctx): Extension<Arc<RequestContext>>,
    Extension(resolved_org): Extension<ResolvedOrganization>,
    Path((slug, member_id)): Path<(ProjectSlug, MemberId)>,
    Json(body): Json<AssignRoleRequestDto>,
) -> Result<StatusCode, ApiError> {
    state
        .mediator
        .dispatch(
            AssignRoleToProjectMember {
                project: ProjectIdentifier::Slug(resolved_org.id, slug),
                member_id,
                role_id: body.role_id,
            },
            &req_ctx,
        )
        .await?;

    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    delete,
    path = "/api/v1/projects/{slug}/members/{member_id}/roles/{role_id}",
    responses(
        (status = 204, description = "Role removed"),
        (status = 404, description = "Member not found"),
    )
)]
pub(crate) async fn remove_role_from_member(
    State(state): State<AppState>,
    Extension(req_ctx): Extension<Arc<RequestContext>>,
    Extension(resolved_org): Extension<ResolvedOrganization>,
    Path((slug, member_id, role_id)): Path<(ProjectSlug, MemberId, ProjectRoleId)>,
) -> Result<StatusCode, ApiError> {
    state
        .mediator
        .dispatch(
            RemoveRoleFromProjectMember {
                project: ProjectIdentifier::Slug(resolved_org.id, slug),
                member_id,
                role_id,
            },
            &req_ctx,
        )
        .await?;

    Ok(StatusCode::NO_CONTENT)
}

// --- Member project memberships ---

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct MemberProjectDto {
    #[serde(rename = "project_name")]
    pub project_name: String,
    #[serde(rename = "project_slug")]
    pub project_slug: String,
    #[serde(rename = "role_id")]
    pub role_id: ProjectRoleId,
    #[serde(rename = "role_name")]
    pub role_name: String,
}

#[utoipa::path(
    get,
    path = "/api/v1/members/{id}/projects",
    responses(
        (status = 200, description = "Member's project memberships", body = Vec<MemberProjectDto>),
    )
)]
pub(crate) async fn list_member_projects(
    State(state): State<AppState>,
    Extension(req_ctx): Extension<Arc<RequestContext>>,
    Path(member_id): Path<MemberId>,
) -> Result<Json<Vec<MemberProjectDto>>, ApiError> {
    let projects = state
        .mediator
        .dispatch(ListMemberProjects { member_id }, &req_ctx)
        .await?;

    let items = projects
        .into_iter()
        .map(|p| MemberProjectDto {
            project_name: p.project_name,
            project_slug: p.project_slug.as_str().to_string(),
            role_id: p.role_id,
            role_name: p.role_name,
        })
        .collect();

    Ok(Json(items))
}

// --- Member access details ---

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct MemberAccessDto {
    #[serde(rename = "id")]
    pub id: MemberId,
    #[serde(rename = "preferred_name")]
    pub preferred_name: String,
    #[serde(rename = "sub")]
    pub sub: String,
    #[serde(rename = "created_at")]
    pub created_at: DateTime<Utc>,
    #[serde(rename = "last_seen")]
    pub last_seen: DateTime<Utc>,
    #[serde(rename = "org_access")]
    pub org_access: OrgAccessDto,
    #[serde(rename = "project_access")]
    pub project_access: Vec<ProjectAccessDto>,
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct OrgAccessDto {
    #[serde(rename = "roles")]
    pub roles: Vec<String>,
    #[serde(rename = "permissions")]
    pub permissions: Vec<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct ProjectAccessDto {
    #[serde(rename = "project_name")]
    pub project_name: String,
    #[serde(rename = "project_slug")]
    pub project_slug: String,
    #[serde(rename = "roles")]
    pub roles: Vec<ProjectRoleAccessDto>,
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct ProjectRoleAccessDto {
    #[serde(rename = "role_name")]
    pub role_name: String,
    #[serde(rename = "permissions")]
    pub permissions: Vec<String>,
}

impl From<MemberAccessResult> for MemberAccessDto {
    fn from(r: MemberAccessResult) -> Self {
        Self {
            id: r.id,
            preferred_name: r.preferred_name,
            sub: r.sub,
            created_at: r.created_at,
            last_seen: r.last_seen,
            org_access: OrgAccessDto {
                roles: r.org_access.roles.into_iter().map(|role| role.to_string()).collect(),
                permissions: r.org_access.permissions.into_iter().map(|p| p.to_string()).collect(),
            },
            project_access: r.project_access.into_iter().map(|pa| ProjectAccessDto {
                project_name: pa.project_name,
                project_slug: pa.project_slug.as_str().to_string(),
                roles: pa.roles.into_iter().map(|ra| ProjectRoleAccessDto {
                    role_name: ra.role_name,
                    permissions: ra.permissions.into_iter().map(|p| p.to_string()).collect(),
                }).collect(),
            }).collect(),
        }
    }
}

#[utoipa::path(
    get,
    path = "/api/v1/members/{id}/access",
    responses(
        (status = 200, description = "Member access details", body = MemberAccessDto),
        (status = 404, description = "Member not found"),
    )
)]
pub(crate) async fn get_member_access(
    State(state): State<AppState>,
    Extension(req_ctx): Extension<Arc<RequestContext>>,
    Extension(resolved_org): Extension<ResolvedOrganization>,
    Path(member_id): Path<MemberId>,
) -> Result<Json<MemberAccessDto>, ApiError> {
    let result = state
        .mediator
        .dispatch(
            GetMemberAccess {
                member_id,
                org_id: resolved_org.id,
            },
            &req_ctx,
        )
        .await?;

    Ok(Json(MemberAccessDto::from(result)))
}
