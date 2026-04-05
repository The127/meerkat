use async_trait::async_trait;
use sqlx::PgPool;

use meerkat_application::error::ApplicationError;
use meerkat_application::ports::issue_repository::IssueRepository;
use meerkat_domain::models::event::EventLevel;
use meerkat_domain::models::issue::{FingerprintHash, Issue, IssueId, IssueIdentifier, IssueNumber, IssueState, IssueStatus};
use meerkat_domain::models::project::ProjectId;
use meerkat_domain::shared::version::Version;

use super::change_buffer::{ChangeTracker, Entry, Identifiable};
use super::error::map_sqlx_error;

impl Identifiable for Issue {
    type Id = IssueId;
    fn id(&self) -> &IssueId { Issue::id(self) }
}

pub(crate) type IssueEntry = Entry<Issue>;

pub struct PgIssueRepository {
    pool: PgPool,
    tracker: ChangeTracker<IssueId, Issue, IssueEntry>,
}

impl PgIssueRepository {
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool,
            tracker: ChangeTracker::new(),
        }
    }

    pub(crate) fn take_entries(&self) -> Vec<IssueEntry> {
        self.tracker.take_entries()
    }

    fn find_in_buffer(&self, identifier: &IssueIdentifier) -> Option<Issue> {
        self.tracker.find_entry(|entry| {
            let issue = entry.entity();
            let matches = match identifier {
                IssueIdentifier::Id(id) => issue.id() == id,
                IssueIdentifier::Fingerprint(project_id, hash) => {
                    issue.project_id() == project_id && issue.fingerprint_hash() == hash
                }
                IssueIdentifier::Number(project_id, number) => {
                    issue.project_id() == project_id && issue.issue_number() == Some(*number)
                }
            };
            if matches { Some(issue.clone()) } else { None }
        })
    }
}

#[derive(sqlx::FromRow)]
struct IssueRow {
    id: sqlx::types::Uuid,
    project_id: sqlx::types::Uuid,
    issue_number: i64,
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
            issue_number: Some(IssueNumber::new(self.issue_number as u64)),
            title: self.title,
            fingerprint_hash: FingerprintHash::new(self.fingerprint_hash).expect("invalid fingerprint_hash in database"),
            status: self.status.parse::<IssueStatus>().expect("invalid status in database"),
            level: self.level.parse::<EventLevel>().expect("invalid level in database"),
            event_count: self.event_count as u64,
            first_seen: self.first_seen,
            last_seen: self.last_seen,
            version: Version::new(self.version as u64),
        })
    }
}

const SELECT_COLUMNS: &str = "id, project_id, issue_number, title, fingerprint_hash, status, level, \
    event_count, first_seen, last_seen, version";

#[async_trait]
impl IssueRepository for PgIssueRepository {
    async fn find(&self, identifier: &IssueIdentifier) -> Result<Issue, ApplicationError> {
        // Check pending entries first — an issue may already be buffered from an earlier
        // find+save in the same UoW (e.g. ingest handler saves, then domain event handler
        // needs the same issue).
        if let Some(issue) = self.find_in_buffer(identifier) {
            self.tracker.track(issue.id().clone(), issue.clone());
            return Ok(issue);
        }

        let row = match identifier {
            IssueIdentifier::Id(id) => {
                sqlx::query_as::<_, IssueRow>(
                    &format!("SELECT {SELECT_COLUMNS} FROM issues WHERE id = $1"),
                )
                .bind(id.as_uuid())
                .fetch_optional(&self.pool)
                .await
                .map_err(map_sqlx_error)?
            }
            IssueIdentifier::Fingerprint(project_id, fingerprint_hash) => {
                sqlx::query_as::<_, IssueRow>(
                    &format!("SELECT {SELECT_COLUMNS} FROM issues WHERE project_id = $1 AND fingerprint_hash = $2"),
                )
                .bind(project_id.as_uuid())
                .bind(fingerprint_hash.as_str())
                .fetch_optional(&self.pool)
                .await
                .map_err(map_sqlx_error)?
            }
            IssueIdentifier::Number(project_id, number) => {
                sqlx::query_as::<_, IssueRow>(
                    &format!("SELECT {SELECT_COLUMNS} FROM issues WHERE project_id = $1 AND issue_number = $2"),
                )
                .bind(project_id.as_uuid())
                .bind(number.value() as i64)
                .fetch_optional(&self.pool)
                .await
                .map_err(map_sqlx_error)?
            }
        }
        .ok_or(ApplicationError::NotFound)?;

        let issue = row.into_issue();
        self.tracker.track(issue.id().clone(), issue.clone());
        Ok(issue)
    }

    fn add(&self, issue: Issue) {
        self.tracker.push(Entry::Added(issue));
    }

    fn save(&self, issue: Issue) {
        self.tracker.save(issue.id().clone(), issue);
    }
}
