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
