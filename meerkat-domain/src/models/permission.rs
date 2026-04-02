#[derive(Debug, Clone, PartialEq, Eq, Hash, strum::Display, strum::EnumString, strum::AsRefStr)]
pub enum Permission {
    #[strum(serialize = "project_read")]
    ProjectRead,
    #[strum(serialize = "project_write")]
    ProjectWrite,
    #[strum(serialize = "project_delete")]
    ProjectDelete,
    #[strum(serialize = "project_manage_members")]
    ProjectManageMembers,
}
