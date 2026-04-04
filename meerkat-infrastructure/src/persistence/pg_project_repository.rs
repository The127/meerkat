use async_trait::async_trait;
use sqlx::PgPool;

use meerkat_application::error::ApplicationError;
use meerkat_application::ports::project_repository::ProjectRepository;
use meerkat_domain::models::organization::OrganizationId;
use meerkat_domain::models::project::{Project, ProjectId, ProjectIdentifier, ProjectSlug, ProjectState};
use meerkat_domain::shared::version::Version;

use super::change_buffer::{BufferEntry, ChangeTracker};
use super::error::map_sqlx_error;

pub(crate) enum ProjectEntry {
    Added(Project),
    Modified {
        entity: Project,
        snapshot: Project,
    },
    Deleted(ProjectId),
}

impl BufferEntry<ProjectId, Project> for ProjectEntry {
    fn id(&self) -> &ProjectId {
        match self {
            ProjectEntry::Added(p) => p.id(),
            ProjectEntry::Modified { entity, .. } => entity.id(),
            ProjectEntry::Deleted(id) => id,
        }
    }

    fn update_entity(&mut self, project: Project) {
        match self {
            ProjectEntry::Added(p) => *p = project,
            ProjectEntry::Modified { entity, .. } => *entity = project,
            ProjectEntry::Deleted(_) => panic!("cannot update a deleted entity"),
        }
    }

    fn make_modified(entity: Project, snapshot: Project) -> Self {
        ProjectEntry::Modified { entity, snapshot }
    }
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

    fn find_in_buffer(&self, identifier: &ProjectIdentifier) -> Option<Project> {
        self.tracker.find_entry(|entry| {
            let project = match entry {
                ProjectEntry::Added(p) | ProjectEntry::Modified { entity: p, .. } => p,
                ProjectEntry::Deleted(_) => return None,
            };
            let matches = match identifier {
                ProjectIdentifier::Id(id) => project.id() == id,
                ProjectIdentifier::Slug(org_id, slug) => {
                    project.organization_id() == org_id && project.slug() == slug
                }
            };
            if matches { Some(project.clone()) } else { None }
        })
    }
}

#[async_trait]
impl ProjectRepository for PgProjectRepository {
    fn add(&self, project: Project) {
        self.tracker.push(ProjectEntry::Added(project));
    }

    fn save(&self, project: Project) {
        self.tracker.save(project.id().clone(), project);
    }

    fn delete(&self, id: ProjectId) {
        self.tracker.remove_snapshot(&id);
        self.tracker.push(ProjectEntry::Deleted(id));
    }

    async fn find(&self, identifier: &ProjectIdentifier) -> Result<Project, ApplicationError> {
        if let Some(project) = self.find_in_buffer(identifier) {
            self.tracker.track(project.id().clone(), project.clone());
            return Ok(project);
        }

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

