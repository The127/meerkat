use async_trait::async_trait;
use sqlx::PgPool;

use meerkat_application::error::ApplicationError;
use meerkat_application::ports::issue_repository::IssueRepository;
use meerkat_domain::models::event::EventLevel;
use meerkat_domain::models::issue::{Issue, IssueId, IssueState, IssueStatus};
use meerkat_domain::models::project::ProjectId;
use meerkat_domain::shared::version::Version;

use super::error::map_sqlx_error;

pub struct PgIssueRepository {
    pool: PgPool,
}

impl PgIssueRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct IssueRow {
    id: sqlx::types::Uuid,
    project_id: sqlx::types::Uuid,
    title: String,
    fingerprint_hash: String,
    status: String,
    level: String,
    event_count: i64,
    first_seen: chrono::DateTime<chrono::Utc>,
    last_seen: chrono::DateTime<chrono::Utc>,
    version: i64,
}

impl IssueRow {
    fn into_issue(self) -> Issue {
        Issue::reconstitute(IssueState {
            id: IssueId::from_uuid(self.id),
            project_id: ProjectId::from_uuid(self.project_id),
            title: self.title,
            fingerprint_hash: self.fingerprint_hash,
            status: self.status.parse::<IssueStatus>().expect("invalid status in database"),
            level: self.level.parse::<EventLevel>().expect("invalid level in database"),
            event_count: self.event_count as u64,
            first_seen: self.first_seen,
            last_seen: self.last_seen,
            version: Version::new(self.version as u64),
        })
    }
}

#[async_trait]
impl IssueRepository for PgIssueRepository {
    async fn find_by_fingerprint(
        &self,
        project_id: &ProjectId,
        fingerprint_hash: &str,
    ) -> Result<Option<Issue>, ApplicationError> {
        let row = sqlx::query_as::<_, IssueRow>(
            "SELECT id, project_id, title, fingerprint_hash, status, level, \
             event_count, first_seen, last_seen, version \
             FROM issues \
             WHERE project_id = $1 AND fingerprint_hash = $2",
        )
        .bind(project_id.as_uuid())
        .bind(fingerprint_hash)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        Ok(row.map(|r| r.into_issue()))
    }

    async fn add(&self, issue: &Issue) -> Result<(), ApplicationError> {
        let now = chrono::Utc::now();

        sqlx::query(
            "INSERT INTO issues (id, project_id, title, fingerprint_hash, status, level, \
             event_count, first_seen, last_seen, version, created_at, updated_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)",
        )
        .bind(issue.id().as_uuid())
        .bind(issue.project_id().as_uuid())
        .bind(issue.title())
        .bind(issue.fingerprint_hash())
        .bind(issue.status().as_ref())
        .bind(issue.level().as_ref())
        .bind(issue.event_count() as i64)
        .bind(issue.first_seen())
        .bind(issue.last_seen())
        .bind(issue.version().as_u64() as i64)
        .bind(now)
        .bind(now)
        .execute(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        Ok(())
    }

    async fn save(&self, issue: &Issue) -> Result<(), ApplicationError> {
        let now = chrono::Utc::now();
        let new_version = issue.version().increment();

        let result = sqlx::query(
            "UPDATE issues SET title = $1, status = $2, level = $3, event_count = $4, \
             first_seen = $5, last_seen = $6, version = $7, updated_at = $8 \
             WHERE id = $9 AND version = $10",
        )
        .bind(issue.title())
        .bind(issue.status().as_ref())
        .bind(issue.level().as_ref())
        .bind(issue.event_count() as i64)
        .bind(issue.first_seen())
        .bind(issue.last_seen())
        .bind(new_version.as_u64() as i64)
        .bind(now)
        .bind(issue.id().as_uuid())
        .bind(issue.version().as_u64() as i64)
        .execute(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        if result.rows_affected() == 0 {
            return Err(ApplicationError::Conflict);
        }

        Ok(())
    }
}
