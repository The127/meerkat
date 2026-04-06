use async_trait::async_trait;
use sqlx::PgPool;

use meerkat_application::error::ApplicationError;
use meerkat_application::ports::project_member_repository::ProjectMemberRepository;
use meerkat_domain::models::member::MemberId;
use meerkat_domain::models::project::ProjectId;
use meerkat_domain::models::project_member::{ProjectMember, ProjectMemberId, ProjectMemberState};
use meerkat_domain::models::project_role::ProjectRoleId;

use super::change_buffer::{ChangeTracker, Entry, Identifiable};
use super::error::map_sqlx_error;

impl Identifiable for ProjectMember {
    type Id = ProjectMemberId;
    fn id(&self) -> &ProjectMemberId { ProjectMember::id(self) }
}

pub(crate) type ProjectMemberEntry = Entry<ProjectMember>;

pub struct PgProjectMemberRepository {
    pool: PgPool,
    tracker: ChangeTracker<ProjectMemberId, ProjectMember, ProjectMemberEntry>,
}

impl PgProjectMemberRepository {
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool,
            tracker: ChangeTracker::new(),
        }
    }

    pub(crate) fn take_entries(&self) -> Vec<ProjectMemberEntry> {
        self.tracker.take_entries()
    }
}

#[async_trait]
impl ProjectMemberRepository for PgProjectMemberRepository {
    fn add(&self, member: ProjectMember) {
        self.tracker.push(Entry::Added(member));
    }

    fn save(&self, member: ProjectMember) {
        self.tracker.save(member.id().clone(), member);
    }

    async fn find_by_project_and_member(
        &self,
        project_id: &ProjectId,
        member_id: &MemberId,
    ) -> Result<Option<ProjectMember>, ApplicationError> {
        #[derive(sqlx::FromRow)]
        struct MemberRow {
            id: sqlx::types::Uuid,
            member_id: sqlx::types::Uuid,
            project_id: sqlx::types::Uuid,
        }

        let row = sqlx::query_as::<_, MemberRow>(
            "SELECT id, member_id, project_id \
             FROM project_members \
             WHERE project_id = $1 AND member_id = $2",
        )
        .bind(project_id.as_uuid())
        .bind(member_id.as_uuid())
        .fetch_optional(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        let Some(row) = row else { return Ok(None) };

        let role_ids: Vec<sqlx::types::Uuid> = sqlx::query_scalar(
            "SELECT role_id FROM project_member_roles WHERE project_member_id = $1",
        )
        .bind(row.id)
        .fetch_all(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        let member = ProjectMember::reconstitute(ProjectMemberState {
            id: ProjectMemberId::from_uuid(row.id),
            member_id: MemberId::from_uuid(row.member_id),
            project_id: ProjectId::from_uuid(row.project_id),
            role_ids: role_ids.into_iter().map(ProjectRoleId::from_uuid).collect(),
        });

        self.tracker.track(member.id().clone(), member.clone());

        Ok(Some(member))
    }
}
