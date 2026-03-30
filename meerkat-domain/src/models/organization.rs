use chrono::{DateTime, Utc};
use meerkat_macros::{uuid_id, slug_id, Reconstitute};
use crate::shared::version::Version;
use crate::shared::change_tracker::ChangeTracker;
use crate::ports::clock::Clock;

uuid_id!(OrganizationId);
slug_id!(OrganizationSlug);

#[derive(Debug, Clone, Reconstitute)]
pub struct Organization {
    id: OrganizationId,
    name: String,
    slug: OrganizationSlug,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    version: Version,
    #[reconstitute_ignore]
    changes: ChangeTracker<OrganizationChange>,
}

#[derive(Debug, Clone)]
pub enum OrganizationChange {
    Created {
        id: OrganizationId,
        name: String,
        slug: OrganizationSlug,
    },
    NameUpdated {
        id: OrganizationId,
        old_name: String,
        new_name: String,
    },
}

#[derive(Debug, thiserror::Error)]
pub enum OrganizationError {
    #[error("organization name must not be empty")]
    EmptyName,
}

impl Organization {
    pub fn new (name: String, slug: OrganizationSlug, clock: &dyn Clock) -> Result<Self, OrganizationError> {
        let name = name.trim();
        if name.is_empty() {
            return Err(OrganizationError::EmptyName);
        }

        let id = OrganizationId::new();
        let name_str = name.to_string();
        let change = OrganizationChange::Created {
            id: id.clone(),
            name: name_str.clone(),
            slug: slug.clone(),
        };

        let now = clock.now();
        let mut changes = ChangeTracker::new();
        changes.record(change);

        Ok(Organization{
            id,
            name: name_str,
            slug,
            created_at: now,
            updated_at: now,
            version: Version::initial(),
            changes,
        })
    }

    pub fn update_name (&mut self, new_name: String, clock: &dyn Clock) -> Result<(), OrganizationError> {
        let new_name = new_name.trim();
        if new_name.is_empty() {
            return Err(OrganizationError::EmptyName);
        }

        if new_name == self.name {
            return Ok(());
        }

        let old_name = self.name.clone();
        let new_name_str = new_name.to_string();

        self.changes.record(OrganizationChange::NameUpdated {
            id: self.id.clone(),
            old_name,
            new_name: new_name_str.clone(),
        });

        self.name = new_name_str;
        self.updated_at = clock.now();

        Ok(())
    }

    pub fn pull_changes(&mut self) -> Vec<OrganizationChange> {
        self.changes.pull_changes()
    }

    pub fn id (&self) -> &OrganizationId {
        &self.id
    }

    pub fn name (&self) -> &str {
        &self.name
    }

    pub fn slug (&self) -> &OrganizationSlug {
        &self.slug
    }

    pub fn created_at (&self) -> &DateTime<Utc> {
        &self.created_at
    }

    pub fn updated_at (&self) -> &DateTime<Utc> {
        &self.updated_at
    }

    pub fn version (&self) -> &Version {
        &self.version
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;
    use crate::ports::clock::MockClock;

    #[test]
    fn given_valid_name_and_slug_organization_creation_should_succeed_and_record_creation_event() {
        // arrange
        let name = "Meerkat Inc.".to_string();
        let slug = OrganizationSlug::from_str("meerkat-inc").unwrap();
        let expected_now = Utc::now();
        let clock = MockClock::new(expected_now);

        // act
        let mut org = Organization::new(name.clone(), slug.clone(), &clock).expect("Failed to create organization");

        // assert
        assert_eq!(org.name(), "Meerkat Inc.");
        assert_eq!(org.slug(), &slug);
        assert_eq!(org.version(), &Version::initial());
        assert!(!org.id().as_uuid().is_nil());
        assert_eq!(org.created_at(), &expected_now);
        assert_eq!(org.updated_at(), &expected_now);

        let changes = org.pull_changes();
        assert_eq!(changes.len(), 1);
        match &changes[0] {
            OrganizationChange::Created { id, name: event_name, slug: event_slug } => {
                assert_eq!(id, org.id());
                assert_eq!(event_name, "Meerkat Inc.");
                assert_eq!(event_slug, &slug);
            },
            _ => panic!("Expected Created change"),
        }
    }

    #[test]
    fn given_an_empty_name_organization_creation_should_fail_with_an_empty_name_error() {
        // arrange
        let name = "  ".to_string();
        let slug = OrganizationSlug::from_str("empty-name").unwrap();
        let now = Utc::now();
        let clock = MockClock::new(now);

        // act
        let result = Organization::new(name, slug, &clock);

        // assert
        match result {
            Err(OrganizationError::EmptyName) => (),
            _ => panic!("Expected EmptyName error, got {:?}", result),
        }
    }

    #[test]
    fn given_a_name_with_extra_spaces_organization_creation_should_trim_the_name() {
        // arrange
        let name = "  Meerkat Inc.  ".to_string();
        let slug = OrganizationSlug::from_str("meerkat-inc").unwrap();
        let now = Utc::now();
        let clock = MockClock::new(now);

        // act
        let org = Organization::new(name, slug, &clock).expect("Failed to create organization");

        // assert
        assert_eq!(org.name(), "Meerkat Inc.");
    }

    #[test]
    fn given_an_existing_organization_updating_its_name_should_succeed_and_record_change_event() {
        // arrange
        let initial_name = "Old Name".to_string();
        let slug = OrganizationSlug::from_str("org-slug").unwrap();
        let initial_now = Utc::now();
        let clock = MockClock::new(initial_now);
        let mut org = Organization::new(initial_name, slug, &clock).unwrap();
        let _ = org.pull_changes(); // clear creation event

        let new_name = "New Name".to_string();
        let updated_now = initial_now + chrono::Duration::hours(1);
        clock.set_now(updated_now);

        // act
        org.update_name(new_name, &clock).expect("Failed to update organization name");

        // assert
        assert_eq!(org.name(), "New Name");
        assert_eq!(org.updated_at(), &updated_now);
        assert_eq!(org.created_at(), &initial_now);
        assert_eq!(org.version(), &Version::initial());

        let changes = org.pull_changes();
        assert_eq!(changes.len(), 1);
        match &changes[0] {
            OrganizationChange::NameUpdated { id, old_name, new_name } => {
                assert_eq!(id, org.id());
                assert_eq!(old_name, "Old Name");
                assert_eq!(new_name, "New Name");
            },
            _ => panic!("Expected NameUpdated change"),
        }
    }


    #[test]
    fn given_an_existing_organization_updating_its_name_to_empty_should_fail() {
        // arrange
        let initial_name = "Old Name".to_string();
        let slug = OrganizationSlug::from_str("org-slug").unwrap();
        let now = Utc::now();
        let clock = MockClock::new(now);
        let mut org = Organization::new(initial_name, slug, &clock).unwrap();

        let empty_name = "  ".to_string();

        // act
        let result = org.update_name(empty_name, &clock);

        // assert
        match result {
            Err(OrganizationError::EmptyName) => (),
            _ => panic!("Expected EmptyName error, got {:?}", result),
        }
    }

    #[test]
    fn given_the_same_name_updating_organization_name_should_do_nothing() {
        // arrange
        let name = "Same Name".to_string();
        let slug = OrganizationSlug::from_str("org-slug").unwrap();
        let now = Utc::now();
        let clock = MockClock::new(now);
        let mut org = Organization::new(name.clone(), slug, &clock).unwrap();
        let _ = org.pull_changes(); // clear creation event

        // act
        org.update_name(name.clone(), &clock).expect("Update should succeed");

        // assert
        assert_eq!(org.name(), &name);
        assert_eq!(org.version(), &Version::initial()); // version should not increment
        assert!(org.pull_changes().is_empty()); // no event should be recorded
    }
}


