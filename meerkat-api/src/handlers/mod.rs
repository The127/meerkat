use meerkat_application::error::ApplicationError;
use vec1::Vec1;

pub mod health;
pub mod members;
pub mod oidc;
pub mod organizations;
pub mod projects;

pub(crate) fn vec1_from_dto(values: Vec<String>, field: &str) -> Result<Vec1<String>, ApplicationError> {
    Vec1::try_from_vec(values)
        .map_err(|_| ApplicationError::Validation(format!("{field} must contain at least one value")))
}