use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use meerkat_application::context::RequestContext;
use meerkat_application::events::ingest::IngestEvent;
use meerkat_domain::models::event::{EventId, EventLevel};

use crate::error::ApiError;
use crate::middleware::key_auth::ProjectContext;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub(crate) struct IngestEventRequestDto {
    #[serde(rename = "message")]
    pub message: String,
    #[serde(rename = "level")]
    pub level: Option<String>,
    #[serde(rename = "platform")]
    pub platform: String,
    #[serde(rename = "timestamp")]
    pub timestamp: Option<DateTime<Utc>>,
    #[serde(rename = "server_name")]
    pub server_name: Option<String>,
    #[serde(rename = "environment")]
    pub environment: Option<String>,
    #[serde(rename = "release")]
    pub release: Option<String>,
    #[serde(rename = "exception_type")]
    pub exception_type: Option<String>,
    #[serde(rename = "exception_value")]
    pub exception_value: Option<String>,
    #[serde(rename = "tags", default)]
    pub tags: Vec<TagDto>,
    #[serde(rename = "extra", default)]
    pub extra: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub(crate) struct TagDto {
    #[serde(rename = "key")]
    pub key: String,
    #[serde(rename = "value")]
    pub value: String,
}

#[derive(Debug, Serialize)]
pub(crate) struct IngestEventResponseDto {
    #[serde(rename = "id")]
    pub id: EventId,
}

pub(crate) async fn ingest_event(
    State(state): State<AppState>,
    project_ctx: ProjectContext,
    Json(body): Json<IngestEventRequestDto>,
) -> Result<(StatusCode, Json<IngestEventResponseDto>), ApiError> {
    let level = match body.level {
        Some(ref l) => l.parse::<EventLevel>().map_err(|_| {
            meerkat_application::error::ApplicationError::Validation(format!("invalid level: {l}"))
        })?,
        None => EventLevel::Error,
    };

    let cmd = IngestEvent {
        project_id: project_ctx.project_id,
        message: body.message,
        level,
        platform: body.platform,
        timestamp: body.timestamp.unwrap_or_else(Utc::now),
        server_name: body.server_name,
        environment: body.environment,
        release: body.release,
        exception_type: body.exception_type,
        exception_value: body.exception_value,
        tags: body.tags.into_iter().map(|t| (t.key, t.value)).collect(),
        extra: body.extra,
    };

    let req_ctx = RequestContext::new(state.context.clone());
    let event_id = state.mediator.dispatch(cmd, &req_ctx).await?;

    Ok((StatusCode::CREATED, Json(IngestEventResponseDto { id: event_id })))
}
