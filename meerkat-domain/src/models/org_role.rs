use crate::models::permission::OrgPermission;

#[derive(Debug, Clone, PartialEq, Eq, strum::Display, strum::EnumString, strum::AsRefStr)]
pub enum OrgRole {
    #[strum(serialize = "owner")]
    Owner,
    #[strum(serialize = "admin")]
    Admin,
    #[strum(serialize = "member")]
    Member,
}

impl OrgRole {
    pub fn permissions(&self) -> Vec<OrgPermission> {
        match self {
            OrgRole::Owner => vec![
                OrgPermission::OrgRename,
                OrgPermission::OrgDelete,
                OrgPermission::OrgManageOidc,
                OrgPermission::OrgManageMembers,
                OrgPermission::OrgCreateProject,
            ],
            OrgRole::Admin => vec![
                OrgPermission::OrgRename,
                OrgPermission::OrgManageOidc,
                OrgPermission::OrgManageMembers,
                OrgPermission::OrgCreateProject,
            ],
            OrgRole::Member => vec![],
        }
    }
}
