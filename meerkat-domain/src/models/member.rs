use chrono::{DateTime, Utc};
use meerkat_macros::{uuid_id, Reconstitute};
use crate::models::organization::OrganizationId;
use crate::ports::clock::Clock;
use crate::shared::change_tracker::ChangeTracker;

uuid_id!(MemberId);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sub(String);

impl Sub {
    pub fn new(value: impl Into<String>) -> Result<Self, MemberError> {
        let value = value.into().trim().to_string();
        if value.is_empty() {
            return Err(MemberError::EmptySub);
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str { &self.0 }
}

impl std::fmt::Display for Sub {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone)]
pub enum MemberIdentifier {
    Id(MemberId),
    OrgSub(OrganizationId, Sub),
}

#[derive(Debug, Clone)]
pub enum MemberChange {
    Created {
        id: MemberId,
        organization_id: OrganizationId,
        sub: Sub,
        preferred_name: String,
    },
    PreferredNameUpdated {
        id: MemberId,
        old_name: String,
        new_name: String,
    },
}

#[derive(Debug, thiserror::Error)]
pub enum MemberError {
    #[error("member sub must not be empty")]
    EmptySub,
    #[error("member preferred name must not be empty")]
    EmptyPreferredName,
}

#[derive(Debug, Clone, Reconstitute)]
pub struct Member {
    id: MemberId,
    organization_id: OrganizationId,
    sub: Sub,
    preferred_name: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    #[reconstitute_ignore]
    changes: ChangeTracker<MemberChange>,
}

impl Member {
    pub fn new(
        organization_id: OrganizationId,
        sub: Sub,
        preferred_name: String,
        clock: &dyn Clock,
    ) -> Result<Self, MemberError> {
        let preferred_name = preferred_name.trim().to_string();
        if preferred_name.is_empty() {
            return Err(MemberError::EmptyPreferredName);
        }

        let now = clock.now();
        let id = MemberId::new();

        let mut changes = ChangeTracker::new();
        changes.record(MemberChange::Created {
            id: id.clone(),
            organization_id: organization_id.clone(),
            sub: sub.clone(),
            preferred_name: preferred_name.clone(),
        });

        Ok(Self {
            id,
            organization_id,
            sub,
            preferred_name,
            created_at: now,
            updated_at: now,
            changes,
        })
    }

    pub fn update_preferred_name(&mut self, new_name: String) -> Result<(), MemberError> {
        let new_name = new_name.trim().to_string();
        if new_name.is_empty() {
            return Err(MemberError::EmptyPreferredName);
        }

        if new_name == self.preferred_name {
            return Ok(());
        }

        let old_name = self.preferred_name.clone();
        self.preferred_name = new_name.clone();
        self.changes.record(MemberChange::PreferredNameUpdated {
            id: self.id.clone(),
            old_name,
            new_name,
        });

        Ok(())
    }

    pub fn pull_changes(&mut self) -> Vec<MemberChange> {
        self.changes.pull_changes()
    }

    pub fn id(&self) -> &MemberId { &self.id }
    pub fn organization_id(&self) -> &OrganizationId { &self.organization_id }
    pub fn sub(&self) -> &Sub { &self.sub }
    pub fn preferred_name(&self) -> &str { &self.preferred_name }
    pub fn created_at(&self) -> &DateTime<Utc> { &self.created_at }
    pub fn updated_at(&self) -> &DateTime<Utc> { &self.updated_at }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ports::clock::MockClock;
    use chrono::Utc;

    fn test_member() -> (Member, MockClock) {
        let clock = MockClock::new(Utc::now());
        let org_id = OrganizationId::new();
        let member = Member::new(org_id, Sub::new("user-123").unwrap(), "Alice".into(), &clock).unwrap();
        (member, clock)
    }

    // --- Sub ---

    #[test]
    fn given_empty_sub_then_sub_creation_fails() {
        // act
        let result = Sub::new("  ");

        // assert
        match result {
            Err(MemberError::EmptySub) => (),
            _ => panic!("Expected EmptySub error, got {:?}", result),
        }
    }

    #[test]
    fn given_valid_sub_then_trims_and_succeeds() {
        // act
        let sub = Sub::new("  user-123  ").unwrap();

        // assert
        assert_eq!(sub.as_str(), "user-123");
    }

    // --- creation ---

    #[test]
    fn given_valid_input_then_creation_succeeds_and_records_created_event() {
        // arrange
        let expected_now = Utc::now();
        let clock = MockClock::new(expected_now);
        let org_id = OrganizationId::new();
        let sub = Sub::new("user-123").unwrap();

        // act
        let mut member = Member::new(org_id.clone(), sub, "Alice".into(), &clock).unwrap();

        // assert
        assert_eq!(member.sub().as_str(), "user-123");
        assert_eq!(member.preferred_name(), "Alice");
        assert_eq!(member.organization_id(), &org_id);
        assert_eq!(member.created_at(), &expected_now);
        assert_eq!(member.updated_at(), &expected_now);

        let changes = member.pull_changes();
        assert_eq!(changes.len(), 1);
        match &changes[0] {
            MemberChange::Created { id, organization_id, sub, preferred_name } => {
                assert_eq!(id, member.id());
                assert_eq!(organization_id, &org_id);
                assert_eq!(sub.as_str(), "user-123");
                assert_eq!(preferred_name, "Alice");
            },
            _ => panic!("Expected Created change"),
        }
    }

    #[test]
    fn given_empty_preferred_name_then_creation_fails() {
        // arrange
        let clock = MockClock::new(Utc::now());

        // act
        let result = Member::new(OrganizationId::new(), Sub::new("user-123").unwrap(), "  ".into(), &clock);

        // assert
        match result {
            Err(MemberError::EmptyPreferredName) => (),
            _ => panic!("Expected EmptyPreferredName error, got {:?}", result),
        }
    }

    #[test]
    fn given_whitespace_preferred_name_then_creation_trims() {
        // arrange
        let clock = MockClock::new(Utc::now());

        // act
        let member = Member::new(OrganizationId::new(), Sub::new("user-123").unwrap(), "  Alice  ".into(), &clock).unwrap();

        // assert
        assert_eq!(member.preferred_name(), "Alice");
    }

    // --- update_preferred_name ---

    #[test]
    fn given_new_name_then_update_succeeds_and_records_change() {
        // arrange
        let (mut member, _) = test_member();
        let _ = member.pull_changes();

        // act
        member.update_preferred_name("Bob".into()).unwrap();

        // assert
        assert_eq!(member.preferred_name(), "Bob");

        let changes = member.pull_changes();
        assert_eq!(changes.len(), 1);
        match &changes[0] {
            MemberChange::PreferredNameUpdated { id, old_name, new_name } => {
                assert_eq!(id, member.id());
                assert_eq!(old_name, "Alice");
                assert_eq!(new_name, "Bob");
            },
            _ => panic!("Expected PreferredNameUpdated change"),
        }
    }

    #[test]
    fn given_same_name_then_update_does_nothing() {
        // arrange
        let (mut member, _) = test_member();
        let _ = member.pull_changes();

        // act
        member.update_preferred_name("Alice".into()).unwrap();

        // assert
        assert!(member.pull_changes().is_empty());
    }

    #[test]
    fn given_empty_name_then_update_fails() {
        // arrange
        let (mut member, _) = test_member();

        // act
        let result = member.update_preferred_name("  ".into());

        // assert
        match result {
            Err(MemberError::EmptyPreferredName) => (),
            _ => panic!("Expected EmptyPreferredName error, got {:?}", result),
        }
    }
}
