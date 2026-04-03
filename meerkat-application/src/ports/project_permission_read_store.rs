use std::collections::HashMap;

use meerkat_domain::models::member::MemberId;
use meerkat_domain::models::permission::ProjectPermission;
use meerkat_domain::models::project::{ProjectId, ProjectSlug};

use crate::error::ApplicationError;

#[async_trait::async_trait]
#[cfg_attr(any(test, feature = "test-utils"), mockall::automock)]
pub trait ProjectPermissionReadStore: Send + Sync {
    async fn get_member_permissions(
        &self,
        member_id: &MemberId,
        project_id: &ProjectId,
    ) -> Result<Vec<ProjectPermission>, ApplicationError>;

    async fn get_all_member_permissions(
        &self,
        member_id: &MemberId,
    ) -> Result<HashMap<ProjectSlug, Vec<ProjectPermission>>, ApplicationError>;
}
