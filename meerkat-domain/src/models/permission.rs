#[derive(Debug, Clone, PartialEq, Eq, Hash, strum::Display, strum::EnumString, strum::AsRefStr)]
pub enum OrgPermission {
    #[strum(serialize = "org_create")]
    OrgCreate,
    #[strum(serialize = "org_rename")]
    OrgRename,
    #[strum(serialize = "org_delete")]
    OrgDelete,
    #[strum(serialize = "org_manage_oidc")]
    OrgManageOidc,
    #[strum(serialize = "org_manage_members")]
    OrgManageMembers,
    #[strum(serialize = "org_create_project")]
    OrgCreateProject,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, strum::Display, strum::EnumString, strum::AsRefStr)]
pub enum ProjectPermission {
    #[strum(serialize = "project_read")]
    ProjectRead,
    #[strum(serialize = "project_write")]
    ProjectWrite,
    #[strum(serialize = "project_delete")]
    ProjectDelete,
    #[strum(serialize = "project_manage_members")]
    ProjectManageMembers,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum EffectivePermission {
    Org(OrgPermission),
    Project(ProjectPermission),
}

impl From<OrgPermission> for EffectivePermission {
    fn from(p: OrgPermission) -> Self { Self::Org(p) }
}

impl From<ProjectPermission> for EffectivePermission {
    fn from(p: ProjectPermission) -> Self { Self::Project(p) }
}
