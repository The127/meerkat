use meerkat_domain::models::oidc_config::OidcConfigError;
use meerkat_domain::models::organization::OrganizationError;
use meerkat_domain::models::project::ProjectError;

#[derive(Debug, thiserror::Error)]
pub enum ApplicationError {
    #[error("validation error: {0}")]
    Validation(String),
    #[error("not found")]
    NotFound,
    #[error("conflict: resource was modified by another request")]
    Conflict,
    #[error("internal error: {0}")]
    Internal(String),
}

impl From<OrganizationError> for ApplicationError {
    fn from(e: OrganizationError) -> Self {
        ApplicationError::Validation(e.to_string())
    }
}

impl From<ProjectError> for ApplicationError {
    fn from(e: ProjectError) -> Self {
        ApplicationError::Validation(e.to_string())
    }
}

impl From<OidcConfigError> for ApplicationError {
    fn from(e: OidcConfigError) -> Self {
        ApplicationError::Validation(e.to_string())
    }
}
