use chrono::{DateTime, Utc};
use meerkat_macros::{uuid_id, slug_id, Reconstitute};
use crate::shared::version::Version;
use crate::shared::change_tracker::ChangeTracker;
use crate::models::organization::OrganizationId;
use crate::ports::clock::Clock;

uuid_id!(ProjectId);
slug_id!(ProjectSlug);

#[derive(Debug, Clone, Reconstitute)]
pub struct Project {
    id: ProjectId,
    organization_id: OrganizationId,
    name: String,
    slug: ProjectSlug,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    version: Version,
    #[reconstitute_ignore]
    changes: ChangeTracker<ProjectChange>,
}

#[derive(Debug, Clone)]
pub enum ProjectChange {
    Created {
        id: ProjectId,
        organization_id: OrganizationId,
        name: String,
        slug: ProjectSlug,
    },
    NameUpdated {
        id: ProjectId,
        old_name: String,
        new_name: String,
    },
}

#[derive(Debug, thiserror::Error)]
pub enum ProjectError {
    #[error("project name must not be empty")]
    EmptyName,
}

impl Project {
    pub fn new(
        organization_id: OrganizationId,
        name: String,
        slug: ProjectSlug,
        clock: &dyn Clock,
    ) -> Result<Self, ProjectError> {
        let name = name.trim();
        if name.is_empty() {
            return Err(ProjectError::EmptyName);
        }

        let id = ProjectId::new();
        let name_str = name.to_string();
        let change = ProjectChange::Created {
            id: id.clone(),
            organization_id: organization_id.clone(),
            name: name_str.clone(),
            slug: slug.clone(),
        };

        let now = clock.now();
        let mut changes = ChangeTracker::new();
        changes.record(change);

        Ok(Project {
            id,
            organization_id,
            name: name_str,
            slug,
            created_at: now,
            updated_at: now,
            version: Version::initial(),
            changes,
        })
    }

    pub fn update_name(&mut self, new_name: String, clock: &dyn Clock) -> Result<(), ProjectError> {
        let new_name = new_name.trim();
        if new_name.is_empty() {
            return Err(ProjectError::EmptyName);
        }

        if new_name == self.name {
            return Ok(());
        }

        let old_name = self.name.clone();
        let new_name_str = new_name.to_string();

        self.changes.record(ProjectChange::NameUpdated {
            id: self.id.clone(),
            old_name,
            new_name: new_name_str.clone(),
        });

        self.name = new_name_str;
        self.updated_at = clock.now();

        Ok(())
    }

    pub fn pull_changes(&mut self) -> Vec<ProjectChange> {
        self.changes.pull_changes()
    }

    pub fn id(&self) -> &ProjectId { &self.id }
    pub fn organization_id(&self) -> &OrganizationId { &self.organization_id }
    pub fn name(&self) -> &str { &self.name }
    pub fn slug(&self) -> &ProjectSlug { &self.slug }
    pub fn created_at(&self) -> &DateTime<Utc> { &self.created_at }
    pub fn updated_at(&self) -> &DateTime<Utc> { &self.updated_at }
    pub fn version(&self) -> &Version { &self.version }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;
    use crate::ports::clock::MockClock;

    #[test]
    fn given_valid_input_project_creation_should_succeed_and_record_creation_event() {
        let org_id = OrganizationId::new();
        let name = "My Project".to_string();
        let slug = ProjectSlug::from_str("my-project").unwrap();
        let now = Utc::now();
        let clock = MockClock::new(now);

        let mut project = Project::new(org_id.clone(), name, slug.clone(), &clock)
            .expect("Failed to create project");

        assert_eq!(project.name(), "My Project");
        assert_eq!(project.slug(), &slug);
        assert_eq!(project.organization_id(), &org_id);
        assert_eq!(project.version(), &Version::initial());
        assert_eq!(project.created_at(), &now);

        let changes = project.pull_changes();
        assert_eq!(changes.len(), 1);
        match &changes[0] {
            ProjectChange::Created { id, organization_id, name, slug: event_slug } => {
                assert_eq!(id, project.id());
                assert_eq!(organization_id, &org_id);
                assert_eq!(name, "My Project");
                assert_eq!(event_slug, &slug);
            }
            _ => panic!("Expected Created change"),
        }
    }

    #[test]
    fn given_empty_name_project_creation_should_fail() {
        let org_id = OrganizationId::new();
        let slug = ProjectSlug::from_str("some-slug").unwrap();
        let clock = MockClock::new(Utc::now());

        let result = Project::new(org_id, "  ".to_string(), slug, &clock);

        match result {
            Err(ProjectError::EmptyName) => (),
            other => panic!("Expected EmptyName error, got {:?}", other),
        }
    }

    #[test]
    fn given_existing_project_updating_name_should_succeed_and_record_change() {
        let org_id = OrganizationId::new();
        let slug = ProjectSlug::from_str("proj").unwrap();
        let initial_now = Utc::now();
        let clock = MockClock::new(initial_now);
        let mut project = Project::new(org_id, "Old Name".to_string(), slug, &clock).unwrap();
        let _ = project.pull_changes();

        let updated_now = initial_now + chrono::Duration::hours(1);
        clock.set_now(updated_now);

        project.update_name("New Name".to_string(), &clock).unwrap();

        assert_eq!(project.name(), "New Name");
        assert_eq!(project.updated_at(), &updated_now);
        assert_eq!(project.version(), &Version::initial());

        let changes = project.pull_changes();
        assert_eq!(changes.len(), 1);
        match &changes[0] {
            ProjectChange::NameUpdated { id, old_name, new_name } => {
                assert_eq!(id, project.id());
                assert_eq!(old_name, "Old Name");
                assert_eq!(new_name, "New Name");
            }
            _ => panic!("Expected NameUpdated change"),
        }
    }

    #[test]
    fn given_same_name_updating_should_do_nothing() {
        let org_id = OrganizationId::new();
        let slug = ProjectSlug::from_str("proj").unwrap();
        let clock = MockClock::new(Utc::now());
        let mut project = Project::new(org_id, "Same".to_string(), slug, &clock).unwrap();
        let _ = project.pull_changes();

        project.update_name("Same".to_string(), &clock).unwrap();

        assert!(project.pull_changes().is_empty());
    }
}
