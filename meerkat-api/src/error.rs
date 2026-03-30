use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, ToSchema)]
pub(crate) struct ErrorBody {
    pub code: String,
    pub message: String,
}