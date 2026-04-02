#[derive(Debug, Clone, PartialEq, Eq, strum::Display, strum::EnumString, strum::AsRefStr)]
pub enum OrgRole {
    #[strum(serialize = "owner")]
    Owner,
    #[strum(serialize = "admin")]
    Admin,
    #[strum(serialize = "member")]
    Member,
}
