use meerkat_domain::models::event::EventError;
use meerkat_domain::models::issue::IssueError;
use meerkat_domain::models::oidc_config::OidcConfigError;
use meerkat_domain::models::organization::OrganizationError;
use meerkat_domain::models::project::ProjectError;
use meerkat_domain::models::project_key::ProjectKeyError;

#[derive(Debug, thiserror::Error)]
pub enum ApplicationError {
    #[error("validation error: {0}")]
    Validation(String),
    #[error("not found")]
    NotFound,
    #[error("conflict: resource was modified by another request")]
    Conflict,
    #[error("unauthorized")]
    Unauthorized,
    #[error("forbidden")]
    Forbidden,
    #[error("rate limited: retry after {retry_after_secs}s")]
    RateLimited { retry_after_secs: u64 },
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

impl From<ProjectKeyError> for ApplicationError {
    fn from(e: ProjectKeyError) -> Self {
        ApplicationError::Validation(e.to_string())
    }
}

impl From<EventError> for ApplicationError {
    fn from(e: EventError) -> Self {
        ApplicationError::Validation(e.to_string())
    }
}

impl From<IssueError> for ApplicationError {
    fn from(e: IssueError) -> Self {
        ApplicationError::Validation(e.to_string())
    }
}
