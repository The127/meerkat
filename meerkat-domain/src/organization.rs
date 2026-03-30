use chrono::{DateTime, Utc};
use meerkat_macros::{uuid_id, slug_id, Reconstitute};
use crate::version::Version;
use meerkat_application::ports::clock::Clock;

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

        let now = clock.now();
        Ok(Organization{
            id: OrganizationId::new(),
            name: name.to_string(),
            slug,
            created_at: now,
            updated_at: now,
            version: Version::initial(),
        })
    }

    pub fn update_name (&mut self, new_name: String, clock: &dyn Clock) -> Result<(), OrganizationError> {
        let new_name = new_name.trim();
        if new_name.is_empty() {
            return Err(OrganizationError::EmptyName);
        }

        self.name = new_name.to_string();
        self.updated_at = clock.now();
        self.version = self.version.increment();

        Ok(())
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
    use meerkat_application::ports::clock::MockClock;

    #[test]
    fn given_valid_name_and_slug_organization_creation_should_succeed() {
        // arrange
        let name = "Meerkat Inc.".to_string();
        let slug = OrganizationSlug::from_str("meerkat-inc").unwrap();
        let expected_now = Utc::now();
        let clock = MockClock::new(expected_now);

        // act
        let org = Organization::new(name.clone(), slug.clone(), &clock).expect("Failed to create organization");

        // assert
        assert_eq!(org.name(), "Meerkat Inc.");
        assert_eq!(org.slug(), &slug);
        assert_eq!(org.version().as_u64(), 1);
        assert!(!org.id().as_uuid().is_nil());
        assert_eq!(org.created_at(), &expected_now);
        assert_eq!(org.updated_at(), &expected_now);
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
    fn given_an_existing_organization_updating_its_name_should_succeed_and_update_metadata() {
        // arrange
        let initial_name = "Old Name".to_string();
        let slug = OrganizationSlug::from_str("org-slug").unwrap();
        let initial_now = Utc::now();
        let clock = MockClock::new(initial_now);
        let mut org = Organization::new(initial_name, slug, &clock).unwrap();

        let new_name = "New Name".to_string();
        let updated_now = initial_now + chrono::Duration::hours(1);
        clock.set_now(updated_now);

        // act
        org.update_name(new_name, &clock).expect("Failed to update organization name");

        // assert
        assert_eq!(org.name(), "New Name");
        assert_eq!(org.updated_at(), &updated_now);
        assert_eq!(org.created_at(), &initial_now);
        assert_eq!(org.version().as_u64(), 2);
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
}


