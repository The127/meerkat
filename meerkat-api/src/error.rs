use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde::Serialize;
use utoipa::ToSchema;

use meerkat_application::error::ApplicationError;
use mediator_rs::MediatorError;
use meerkat_application::ports::error_observer::{ErrorReport, ErrorSeverity};

#[derive(Debug, Clone, Serialize, ToSchema)]
pub(crate) struct ErrorDto {
    #[serde(rename = "code")]
    pub code: String,
    #[serde(rename = "message")]
    pub message: String,
}

pub(crate) struct ApiError(ApiErrorKind);

enum ApiErrorKind {
    Application(ApplicationError),
    Mediator(MediatorError<ApplicationError>),
}

impl From<ApplicationError> for ApiError {
    fn from(err: ApplicationError) -> Self {
        Self(ApiErrorKind::Application(err))
    }
}

impl From<MediatorError<ApplicationError>> for ApiError {
    fn from(err: MediatorError<ApplicationError>) -> Self {
        Self(ApiErrorKind::Mediator(err))
    }
}

impl ApiError {
    fn retry_after_secs(&self) -> Option<u64> {
        match &self.0 {
            ApiErrorKind::Application(ApplicationError::RateLimited { retry_after_secs }) => {
                Some(*retry_after_secs)
            }
            ApiErrorKind::Mediator(MediatorError::HandlerError(ApplicationError::RateLimited {
                retry_after_secs,
            })) => Some(*retry_after_secs),
            _ => None,
        }
    }

    fn into_parts(self) -> (StatusCode, &'static str, String, String, ErrorSeverity) {
        match self.0 {
            ApiErrorKind::Application(ref err) => Self::application_error_parts(err),
            ApiErrorKind::Mediator(MediatorError::HandlerError(ref err)) => {
                Self::application_error_parts(err)
            }
            ApiErrorKind::Mediator(MediatorError::NoHandlerRegistered(type_id)) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal_error",
                format!("no handler registered for {:?}", type_id),
                "an unexpected error occurred".to_string(),
                ErrorSeverity::Critical,
            ),
        }
    }

    fn application_error_parts(err: &ApplicationError) -> (StatusCode, &'static str, String, String, ErrorSeverity) {
        match err {
            ApplicationError::RateLimited { retry_after_secs } => (
                StatusCode::TOO_MANY_REQUESTS,
                "rate_limited",
                format!("rate limited: retry after {}s", retry_after_secs),
                format!("too many requests, retry after {}s", retry_after_secs),
                ErrorSeverity::Warning,
            ),
            ApplicationError::Validation(msg) => (
                StatusCode::BAD_REQUEST,
                "validation_error",
                msg.clone(),
                msg.clone(),
                ErrorSeverity::Warning,
            ),
            ApplicationError::NotFound => (
                StatusCode::NOT_FOUND,
                "not_found",
                "resource not found".to_string(),
                "resource not found".to_string(),
                ErrorSeverity::Warning,
            ),
            ApplicationError::Unauthorized => (
                StatusCode::UNAUTHORIZED,
                "unauthorized",
                "unauthorized".to_string(),
                "unauthorized".to_string(),
                ErrorSeverity::Warning,
            ),
            ApplicationError::Forbidden => (
                StatusCode::FORBIDDEN,
                "forbidden",
                "forbidden".to_string(),
                "insufficient permissions".to_string(),
                ErrorSeverity::Warning,
            ),
            ApplicationError::Conflict => (
                StatusCode::CONFLICT,
                "conflict",
                "resource was modified by another request".to_string(),
                "resource was modified by another request".to_string(),
                ErrorSeverity::Warning,
            ),
            ApplicationError::Internal(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal_error",
                msg.clone(),
                "an unexpected error occurred".to_string(),
                ErrorSeverity::Error,
            ),
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let retry_after = self.retry_after_secs();
        let (status, code, internal_message, client_message, severity) = self.into_parts();

        let report = ErrorReport {
            message: internal_message,
            severity,
            source: "api".to_string(),
        };

        let mut response = (status, Json(ErrorDto {
            code: code.to_string(),
            message: client_message,
        })).into_response();

        if let Some(secs) = retry_after {
            response.headers_mut().insert(
                axum::http::header::RETRY_AFTER,
                axum::http::HeaderValue::from(secs),
            );
        }

        response.extensions_mut().insert(report);
        response
    }
}
