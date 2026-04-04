use async_trait::async_trait;
use sqlx::PgPool;

use meerkat_application::error::ApplicationError;
use meerkat_application::ports::project_repository::ProjectRepository;
use meerkat_domain::models::organization::OrganizationId;
use meerkat_domain::models::project::{Project, ProjectId, ProjectIdentifier, ProjectSlug, ProjectState};
use meerkat_domain::shared::version::Version;

use super::change_buffer::ChangeTracker;
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
    tracker: ChangeTracker<ProjectId, Project, ProjectEntry>,
}

impl PgProjectRepository {
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool,
            tracker: ChangeTracker::new(),
        }
    }

    pub(crate) fn take_entries(&self) -> Vec<ProjectEntry> {
        self.tracker.take_entries()
    }
}

#[async_trait]
impl ProjectRepository for PgProjectRepository {
    fn add(&self, project: Project) {
        self.tracker.push(ProjectEntry::Added(project));
    }

    fn save(&self, project: Project) {
        let snapshot = self.tracker.take_snapshot(project.id());
        self.tracker.push(ProjectEntry::Modified { entity: project, snapshot });
    }

    fn delete(&self, id: ProjectId) {
        self.tracker.remove_snapshot(&id);
        self.tracker.push(ProjectEntry::Deleted(id));
    }

    async fn find(&self, identifier: &ProjectIdentifier) -> Result<Project, ApplicationError> {
        let row = match identifier {
            ProjectIdentifier::Id(id) => {
                sqlx::query_as::<_, ProjectRow>(
                    "SELECT id, organization_id, name, slug, version \
                     FROM projects WHERE id = $1",
                )
                .bind(id.as_uuid())
                .fetch_optional(&self.pool)
                .await
            }
            ProjectIdentifier::Slug(org_id, slug) => {
                sqlx::query_as::<_, ProjectRow>(
                    "SELECT id, organization_id, name, slug, version \
                     FROM projects WHERE organization_id = $1 AND slug = $2",
                )
                .bind(org_id.as_uuid())
                .bind(slug.as_str())
                .fetch_optional(&self.pool)
                .await
            }
        }
        .map_err(map_sqlx_error)?
        .ok_or(ApplicationError::NotFound)?;

        let project = Project::reconstitute(ProjectState {
            id: ProjectId::from_uuid(row.id),
            organization_id: OrganizationId::from_uuid(row.organization_id),
            name: row.name,
            slug: ProjectSlug::new(row.slug).expect("invalid slug in database"),
            version: Version::new(row.version as u64),
        });

        self.tracker.track(project.id().clone(), project.clone());

        Ok(project)
    }
}

#[derive(sqlx::FromRow)]
struct ProjectRow {
    id: sqlx::types::Uuid,
    organization_id: sqlx::types::Uuid,
    name: String,
    slug: String,
    version: i64,
}

