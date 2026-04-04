use axum::extract::FromRequestParts;
use axum::http::StatusCode;
use axum::http::request::Parts;
use axum::response::{IntoResponse, Response};
use axum::Json;

use meerkat_domain::models::project::ProjectId;
use meerkat_domain::models::project_key::KeyToken;

use crate::error::ErrorDto;
use crate::state::AppState;

#[derive(Debug, Clone)]
pub(crate) struct ProjectContext {
    pub project_id: ProjectId,
    pub key_token: KeyToken,
}

impl FromRequestParts<AppState> for ProjectContext {
    type Rejection = Response;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let token = parts
            .headers
            .get("X-Meerkat-Key")
            .and_then(|v| v.to_str().ok())
            .ok_or_else(|| unauthorized("missing X-Meerkat-Key header"))?;

        let key = state
            .project_key_read_store
            .find_by_token(token)
            .await
            .map_err(|_| internal_error())?
            .ok_or_else(|| unauthorized("invalid project key"))?;

        Ok(ProjectContext {
            project_id: key.project_id,
            key_token: KeyToken::new(token).map_err(|_| unauthorized("invalid project key"))?,
        })
    }
}

fn unauthorized(message: &str) -> Response {
    (
        StatusCode::UNAUTHORIZED,
        Json(ErrorDto {
            code: "unauthorized".to_string(),
            message: message.to_string(),
        }),
    )
        .into_response()
}

fn internal_error() -> Response {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ErrorDto {
            code: "internal_error".to_string(),
            message: "an unexpected error occurred".to_string(),
        }),
    )
        .into_response()
}
