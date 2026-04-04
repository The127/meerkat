use meerkat_application::error::ApplicationError;
use meerkat_domain::models::oidc_config::RoleValues;
use vec1::Vec1;

pub mod health;
pub mod members;
pub mod oidc;
pub mod oidc_admin;
pub mod organizations;
pub mod project_keys;
pub mod projects;
pub mod team;

pub(crate) fn vec1_from_dto(values: Vec<String>, field: &str) -> Result<Vec1<String>, ApplicationError> {
    Vec1::try_from_vec(values)
        .map_err(|_| ApplicationError::Validation(format!("{field} must contain at least one value")))
}

pub(crate) fn role_values_from_dto(dto: organizations::RoleValuesDto) -> Result<RoleValues, ApplicationError> {
    let owner = vec1_from_dto(dto.owner, "role_values.owner")?;
    let admin = vec1_from_dto(dto.admin, "role_values.admin")?;
    let member = vec1_from_dto(dto.member, "role_values.member")?;
    Ok(RoleValues::new(owner, admin, member))
}