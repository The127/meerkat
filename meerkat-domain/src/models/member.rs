use chrono::{DateTime, Utc};
use meerkat_macros::{uuid_id, Reconstitute};
use crate::models::organization::OrganizationId;
use crate::ports::clock::Clock;

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

        Ok(Self {
            id,
            organization_id,
            sub,
            preferred_name,
            created_at: now,
            updated_at: now,
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

        self.preferred_name = new_name;
        Ok(())
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
    fn given_valid_input_then_creation_succeeds() {
        // arrange
        let expected_now = Utc::now();
        let clock = MockClock::new(expected_now);
        let org_id = OrganizationId::new();
        let sub = Sub::new("user-123").unwrap();

        // act
        let member = Member::new(org_id.clone(), sub, "Alice".into(), &clock).unwrap();

        // assert
        assert_eq!(member.sub().as_str(), "user-123");
        assert_eq!(member.preferred_name(), "Alice");
        assert_eq!(member.organization_id(), &org_id);
        assert_eq!(member.created_at(), &expected_now);
        assert_eq!(member.updated_at(), &expected_now);
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
    fn given_new_name_then_update_succeeds() {
        // arrange
        let (mut member, _) = test_member();

        // act
        member.update_preferred_name("Bob".into()).unwrap();

        // assert
        assert_eq!(member.preferred_name(), "Bob");
    }

    #[test]
    fn given_same_name_then_update_does_nothing() {
        // arrange
        let (mut member, _) = test_member();

        // act
        member.update_preferred_name("Alice".into()).unwrap();

        // assert
        assert_eq!(member.preferred_name(), "Alice");
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
