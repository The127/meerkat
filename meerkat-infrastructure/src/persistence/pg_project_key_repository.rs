use async_trait::async_trait;
use sqlx::PgPool;

use meerkat_application::error::ApplicationError;
use meerkat_application::ports::project_key_repository::ProjectKeyRepository;
use meerkat_domain::models::project::ProjectId;
use meerkat_domain::models::project_key::{
    KeyToken, ProjectKey, ProjectKeyId, ProjectKeyState, ProjectKeyStatus, RateLimit,
};
use meerkat_domain::shared::version::Version;

use super::change_buffer::{ChangeTracker, Entry, Identifiable};
use super::error::map_sqlx_error;

impl Identifiable for ProjectKey {
    type Id = ProjectKeyId;
    fn id(&self) -> &ProjectKeyId { ProjectKey::id(self) }
}

pub(crate) type ProjectKeyEntry = Entry<ProjectKey>;

pub struct PgProjectKeyRepository {
    pool: PgPool,
    tracker: ChangeTracker<ProjectKeyId, ProjectKey, ProjectKeyEntry>,
}

impl PgProjectKeyRepository {
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool,
            tracker: ChangeTracker::new(),
        }
    }

    pub(crate) fn take_entries(&self) -> Vec<ProjectKeyEntry> {
        self.tracker.take_entries()
    }

    fn find_in_buffer(&self, id: &ProjectKeyId) -> Option<ProjectKey> {
        self.tracker.find_entry(|entry| {
            let key = entry.entity();
            if key.id() == id { Some(key.clone()) } else { None }
        })
    }
}

#[async_trait]
impl ProjectKeyRepository for PgProjectKeyRepository {
    fn add(&self, key: ProjectKey) {
        self.tracker.push(Entry::Added(key));
    }

    fn save(&self, key: ProjectKey) {
        self.tracker.save(key.id().clone(), key);
    }

    async fn find(&self, id: &ProjectKeyId) -> Result<ProjectKey, ApplicationError> {
        if let Some(key) = self.find_in_buffer(id) {
            self.tracker.track(key.id().clone(), key.clone());
            return Ok(key);
        }

        let row = sqlx::query_as::<_, ProjectKeyRow>(
            "SELECT id, project_id, key_token, label, status, rate_limit, version \
             FROM project_keys WHERE id = $1",
        )
        .bind(id.as_uuid())
        .fetch_optional(&self.pool)
        .await
        .map_err(map_sqlx_error)?
        .ok_or(ApplicationError::NotFound)?;

        let key = ProjectKey::reconstitute(ProjectKeyState {
            id: ProjectKeyId::from_uuid(row.id),
            project_id: ProjectId::from_uuid(row.project_id),
            key_token: KeyToken::new(row.key_token).expect("invalid key_token in database"),
            label: row.label,
            status: row.status.parse::<ProjectKeyStatus>().expect("invalid status in database"),
            rate_limit: row.rate_limit.map(|v| RateLimit::new(v as u64).expect("invalid rate_limit in database")),
            version: Version::new(row.version as u64),
        });

        self.tracker.track(key.id().clone(), key.clone());

        Ok(key)
    }
}

#[derive(sqlx::FromRow)]
struct ProjectKeyRow {
    id: sqlx::types::Uuid,
    project_id: sqlx::types::Uuid,
    key_token: String,
    label: String,
    status: String,
    rate_limit: Option<i64>,
    version: i64,
}
