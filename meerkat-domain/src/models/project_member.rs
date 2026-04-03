use chrono::{DateTime, Utc};
use meerkat_macros::uuid_id;
use crate::models::member::{MemberId, Sub};
use crate::models::project::ProjectId;
use crate::models::project_role::ProjectRoleId;
use crate::ports::clock::Clock;

uuid_id!(ProjectMemberId);

#[derive(Debug, Clone)]
pub enum ProjectMemberIdentifier {
    Id(ProjectMemberId),
    ProjectSub(ProjectId, Sub),
}

#[derive(Debug, Clone)]
pub struct ProjectMember {
    id: ProjectMemberId,
    member_id: MemberId,
    project_id: ProjectId,
    role_ids: Vec<ProjectRoleId>,
    created_at: DateTime<Utc>,
}

impl ProjectMember {
    pub fn new(
        member_id: MemberId,
        project_id: ProjectId,
        role_ids: Vec<ProjectRoleId>,
        clock: &dyn Clock,
    ) -> Self {
        let id = ProjectMemberId::new();

        Self {
            id,
            member_id,
            project_id,
            role_ids,
            created_at: clock.now(),
        }
    }

    pub fn assign_role(&mut self, role_id: ProjectRoleId) {
        if self.role_ids.contains(&role_id) {
            return;
        }

        self.role_ids.push(role_id);
    }

    pub fn remove_role(&mut self, role_id: &ProjectRoleId) {
        self.role_ids.retain(|r| r != role_id);
    }

    pub fn id(&self) -> &ProjectMemberId { &self.id }
    pub fn member_id(&self) -> &MemberId { &self.member_id }
    pub fn project_id(&self) -> &ProjectId { &self.project_id }
    pub fn role_ids(&self) -> &[ProjectRoleId] { &self.role_ids }
    pub fn created_at(&self) -> &DateTime<Utc> { &self.created_at }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ports::clock::MockClock;
    use chrono::Utc;

    #[test]
    fn given_valid_input_then_creation_succeeds() {
        // arrange
        let clock = MockClock::new(Utc::now());
        let member_id = MemberId::new();
        let project_id = ProjectId::new();
        let role_id = ProjectRoleId::new();

        // act
        let pm = ProjectMember::new(member_id.clone(), project_id.clone(), vec![role_id.clone()], &clock);

        // assert
        assert_eq!(pm.member_id(), &member_id);
        assert_eq!(pm.project_id(), &project_id);
        assert_eq!(pm.role_ids(), &[role_id]);
    }

    #[test]
    fn given_new_role_then_assign_adds_it() {
        // arrange
        let clock = MockClock::new(Utc::now());
        let mut pm = ProjectMember::new(MemberId::new(), ProjectId::new(), vec![], &clock);
        let role_id = ProjectRoleId::new();

        // act
        pm.assign_role(role_id.clone());

        // assert
        assert_eq!(pm.role_ids(), &[role_id]);
    }

    #[test]
    fn given_duplicate_role_then_assign_is_idempotent() {
        // arrange
        let clock = MockClock::new(Utc::now());
        let role_id = ProjectRoleId::new();
        let mut pm = ProjectMember::new(MemberId::new(), ProjectId::new(), vec![role_id.clone()], &clock);

        // act
        pm.assign_role(role_id.clone());

        // assert
        assert_eq!(pm.role_ids(), &[role_id]);
    }

    #[test]
    fn given_existing_role_then_remove_removes_it() {
        // arrange
        let clock = MockClock::new(Utc::now());
        let role_id = ProjectRoleId::new();
        let mut pm = ProjectMember::new(MemberId::new(), ProjectId::new(), vec![role_id.clone()], &clock);

        // act
        pm.remove_role(&role_id);

        // assert
        assert!(pm.role_ids().is_empty());
    }

    #[test]
    fn given_nonexistent_role_then_remove_is_idempotent() {
        // arrange
        let clock = MockClock::new(Utc::now());
        let existing = ProjectRoleId::new();
        let mut pm = ProjectMember::new(MemberId::new(), ProjectId::new(), vec![existing.clone()], &clock);

        // act
        pm.remove_role(&ProjectRoleId::new());

        // assert
        assert_eq!(pm.role_ids(), &[existing]);
    }
}
