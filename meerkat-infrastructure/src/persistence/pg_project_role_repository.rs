use async_trait::async_trait;
use sqlx::PgPool;
use vec1::Vec1;

use meerkat_application::error::ApplicationError;
use meerkat_application::ports::project_role_repository::ProjectRoleRepository;
use meerkat_domain::models::permission::ProjectPermission;
use meerkat_domain::models::project::ProjectId;
use meerkat_domain::models::project_role::{
    ProjectRole, ProjectRoleId, ProjectRoleSlug, ProjectRoleState,
};

use super::change_buffer::{ChangeTracker, DeletableEntry, Identifiable};
use super::error::map_sqlx_error;

impl Identifiable for ProjectRole {
    type Id = ProjectRoleId;
    fn id(&self) -> &ProjectRoleId { ProjectRole::id(self) }
}

pub(crate) type ProjectRoleEntry = DeletableEntry<ProjectRole>;

pub struct PgProjectRoleRepository {
    pool: PgPool,
    tracker: ChangeTracker<ProjectRoleId, ProjectRole, ProjectRoleEntry>,
}

impl PgProjectRoleRepository {
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool,
            tracker: ChangeTracker::new(),
        }
    }

    pub(crate) fn take_entries(&self) -> Vec<ProjectRoleEntry> {
        self.tracker.take_entries()
    }

    fn find_in_buffer(&self, id: &ProjectRoleId) -> Option<ProjectRole> {
        self.tracker.find_entry(|entry| {
            let role = entry.entity()?;
            if role.id() == id { Some(role.clone()) } else { None }
        })
    }
}

#[async_trait]
impl ProjectRoleRepository for PgProjectRoleRepository {
    fn add(&self, role: ProjectRole) {
        self.tracker.push(DeletableEntry::Added(role));
    }

    fn save(&self, role: ProjectRole) {
        self.tracker.save(role.id().clone(), role);
    }

    fn delete(&self, id: ProjectRoleId) {
        self.tracker.remove_snapshot(&id);
        self.tracker.push(DeletableEntry::Deleted(id));
    }

    async fn find(&self, id: &ProjectRoleId) -> Result<ProjectRole, ApplicationError> {
        if let Some(role) = self.find_in_buffer(id) {
            self.tracker.track(role.id().clone(), role.clone());
            return Ok(role);
        }

        #[derive(sqlx::FromRow)]
        struct RoleRow {
            id: sqlx::types::Uuid,
            project_id: sqlx::types::Uuid,
            name: String,
            slug: String,
            is_default: bool,
            permissions: Vec<String>,
        }

        let row = sqlx::query_as::<_, RoleRow>(
            "SELECT r.id, r.project_id, r.name, r.slug, r.is_default, \
                    COALESCE(array_agg(p.permission ORDER BY p.permission) \
                             FILTER (WHERE p.permission IS NOT NULL), '{}') AS permissions \
             FROM project_roles r \
             LEFT JOIN project_role_permissions p ON p.role_id = r.id \
             WHERE r.id = $1 \
             GROUP BY r.id, r.project_id, r.name, r.slug, r.is_default",
        )
        .bind(id.as_uuid())
        .fetch_optional(&self.pool)
        .await
        .map_err(map_sqlx_error)?
        .ok_or(ApplicationError::NotFound)?;

        let permissions: Vec<ProjectPermission> = row.permissions
            .into_iter()
            .filter_map(|s| s.parse::<ProjectPermission>().ok())
            .collect();

        let permissions = Vec1::try_from_vec(permissions)
            .map_err(|_| ApplicationError::Internal("role has no permissions in database".into()))?;

        let role = ProjectRole::reconstitute(ProjectRoleState {
            id: ProjectRoleId::from_uuid(row.id),
            project_id: ProjectId::from_uuid(row.project_id),
            name: row.name,
            slug: ProjectRoleSlug::new(row.slug).expect("invalid slug in database"),
            permissions,
            is_default: row.is_default,
        });

        self.tracker.track(role.id().clone(), role.clone());

        Ok(role)
    }
}
