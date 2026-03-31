use meerkat_application::error::ApplicationError;

pub(crate) fn map_sqlx_error(err: sqlx::Error) -> ApplicationError {
    if let sqlx::Error::Database(ref db_err) = err
        && db_err.code().as_deref() == Some("23505")
    {
        return ApplicationError::Conflict;
    }
    ApplicationError::Internal(err.to_string())
}
