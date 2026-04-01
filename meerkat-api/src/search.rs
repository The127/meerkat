use serde::Deserialize;
use utoipa::{IntoParams, ToSchema};

#[derive(Debug, Deserialize, IntoParams, ToSchema)]
pub(crate) struct SearchQueryDto {
    #[serde(rename = "search")]
    pub search: Option<String>,
}
