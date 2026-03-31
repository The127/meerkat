use std::collections::HashMap;
use std::sync::Mutex;

use async_trait::async_trait;
use sqlx::PgPool;

use meerkat_application::error::ApplicationError;
use meerkat_application::ports::project_repository::ProjectRepository;
use meerkat_domain::models::organization::OrganizationId;
use meerkat_domain::models::project::{Project, ProjectId, ProjectSlug, ProjectState};
use meerkat_domain::shared::version::Version;

use super::error::map_sqlx_error;

pub(crate) enum ProjectEntry {
    Added(Project),
    Modified {
        entity: Project,
        snapshot: Project,
    },
    Deleted(ProjectId),
}

pub struct PgProjectRepository {
    pool: PgPool,
    snapshots: Mutex<HashMap<ProjectId, Project>>,
    buffer: Mutex<Vec<ProjectEntry>>,
}

impl PgProjectRepository {
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool,
            snapshots: Mutex::new(HashMap::new()),
            buffer: Mutex::new(Vec::new()),
        }
    }

    pub(crate) fn take_entries(&self) -> Vec<ProjectEntry> {
        std::mem::take(&mut *self.buffer.lock().unwrap())
    }
}

#[async_trait]
impl ProjectRepository for PgProjectRepository {
    fn add(&self, project: Project) {
        self.buffer.lock().unwrap().push(ProjectEntry::Added(project));
    }

    fn save(&self, project: Project) {
        let snapshot = self
            .snapshots
            .lock()
            .unwrap()
            .remove(project.id())
            .expect("save called without prior find_by_id");

        self.buffer
            .lock()
            .unwrap()
            .push(ProjectEntry::Modified { entity: project, snapshot });
    }

    fn delete(&self, id: ProjectId) {
        self.snapshots.lock().unwrap().remove(&id);
        self.buffer.lock().unwrap().push(ProjectEntry::Deleted(id));
    }

    async fn find_by_id(&self, id: &ProjectId) -> Result<Project, ApplicationError> {
        let row = sqlx::query_as::<_, ProjectRow>(
            "SELECT id, organization_id, name, slug, created_at, updated_at, version \
             FROM projects WHERE id = $1",
        )
        .bind(id.as_uuid())
        .fetch_optional(&self.pool)
        .await
        .map_err(map_sqlx_error)?
        .ok_or(ApplicationError::NotFound)?;

        let project = Project::reconstitute(ProjectState {
            id: ProjectId::from_uuid(row.id),
            organization_id: OrganizationId::from_uuid(row.organization_id),
            name: row.name,
            slug: ProjectSlug::new(row.slug).expect("invalid slug in database"),
            created_at: row.created_at,
            updated_at: row.updated_at,
            version: Version::new(row.version as u64),
        });

        self.snapshots
            .lock()
            .unwrap()
            .insert(id.clone(), project.clone());

        Ok(project)
    }
}

#[derive(sqlx::FromRow)]
struct ProjectRow {
    id: sqlx::types::Uuid,
    organization_id: sqlx::types::Uuid,
    name: String,
    slug: String,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
    version: i64,
}

