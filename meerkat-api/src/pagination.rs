use serde::Deserialize;
use utoipa::{IntoParams, ToSchema};

#[derive(Debug, Deserialize, IntoParams, ToSchema)]
pub(crate) struct PaginationQueryDto {
    #[serde(rename = "limit")]
    #[param(minimum = 1, maximum = 100)]
    pub limit: Option<i64>,
    #[serde(rename = "offset")]
    #[param(minimum = 0)]
    pub offset: Option<i64>,
}

impl PaginationQueryDto {
    pub fn limit(&self) -> i64 {
        self.limit.unwrap_or(20).clamp(1, 100)
    }

    pub fn offset(&self) -> i64 {
        self.offset.unwrap_or(0).max(0)
    }
}
