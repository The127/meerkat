use chrono::{DateTime, Utc};
use meerkat_macros::uuid_id;
use crate::models::member::{MemberId, Sub};
use crate::models::project::ProjectId;
use crate::models::project_role::ProjectRoleId;
use crate::ports::clock::Clock;
use crate::shared::change_tracker::ChangeTracker;

uuid_id!(ProjectMemberId);

#[derive(Debug, Clone)]
pub enum ProjectMemberIdentifier {
    Id(ProjectMemberId),
    ProjectSub(ProjectId, Sub),
}

#[derive(Debug, Clone)]
pub enum ProjectMemberChange {
    Created {
        id: ProjectMemberId,
        member_id: MemberId,
        project_id: ProjectId,
        role_ids: Vec<ProjectRoleId>,
    },
    RoleAssigned {
        id: ProjectMemberId,
        role_id: ProjectRoleId,
    },
    RoleRemoved {
        id: ProjectMemberId,
        role_id: ProjectRoleId,
    },
}

#[derive(Debug, Clone)]
pub struct ProjectMember {
    id: ProjectMemberId,
    member_id: MemberId,
    project_id: ProjectId,
    role_ids: Vec<ProjectRoleId>,
    created_at: DateTime<Utc>,
    changes: ChangeTracker<ProjectMemberChange>,
}

impl ProjectMember {
    pub fn new(
        member_id: MemberId,
        project_id: ProjectId,
        role_ids: Vec<ProjectRoleId>,
        clock: &dyn Clock,
    ) -> Self {
        let id = ProjectMemberId::new();

        let mut changes = ChangeTracker::new();
        changes.record(ProjectMemberChange::Created {
            id: id.clone(),
            member_id: member_id.clone(),
            project_id: project_id.clone(),
            role_ids: role_ids.clone(),
        });

        Self {
            id,
            member_id,
            project_id,
            role_ids,
            created_at: clock.now(),
            changes,
        }
    }

    pub fn assign_role(&mut self, role_id: ProjectRoleId) {
        if self.role_ids.contains(&role_id) {
            return;
        }

        self.role_ids.push(role_id.clone());
        self.changes.record(ProjectMemberChange::RoleAssigned {
            id: self.id.clone(),
            role_id,
        });
    }

    pub fn remove_role(&mut self, role_id: &ProjectRoleId) {
        if !self.role_ids.contains(role_id) {
            return;
        }

        self.role_ids.retain(|r| r != role_id);
        self.changes.record(ProjectMemberChange::RoleRemoved {
            id: self.id.clone(),
            role_id: role_id.clone(),
        });
    }

    pub fn pull_changes(&mut self) -> Vec<ProjectMemberChange> {
        self.changes.pull_changes()
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
    fn given_valid_input_then_creation_succeeds_and_records_created_event() {
        // arrange
        let clock = MockClock::new(Utc::now());
        let member_id = MemberId::new();
        let project_id = ProjectId::new();
        let role_id = ProjectRoleId::new();

        // act
        let mut pm = ProjectMember::new(member_id.clone(), project_id.clone(), vec![role_id.clone()], &clock);

        // assert
        assert_eq!(pm.member_id(), &member_id);
        assert_eq!(pm.project_id(), &project_id);
        assert_eq!(pm.role_ids(), &[role_id.clone()]);

        let changes = pm.pull_changes();
        assert_eq!(changes.len(), 1);
        match &changes[0] {
            ProjectMemberChange::Created { id, member_id: mid, project_id: pid, role_ids } => {
                assert_eq!(id, pm.id());
                assert_eq!(mid, &member_id);
                assert_eq!(pid, &project_id);
                assert_eq!(role_ids, &[role_id]);
            },
            _ => panic!("Expected Created change"),
        }
    }

    #[test]
    fn given_new_role_then_assign_records_role_assigned_event() {
        // arrange
        let clock = MockClock::new(Utc::now());
        let mut pm = ProjectMember::new(MemberId::new(), ProjectId::new(), vec![], &clock);
        let _ = pm.pull_changes();
        let role_id = ProjectRoleId::new();

        // act
        pm.assign_role(role_id.clone());

        // assert
        assert_eq!(pm.role_ids(), &[role_id.clone()]);

        let changes = pm.pull_changes();
        assert_eq!(changes.len(), 1);
        match &changes[0] {
            ProjectMemberChange::RoleAssigned { id, role_id: rid } => {
                assert_eq!(id, pm.id());
                assert_eq!(rid, &role_id);
            },
            _ => panic!("Expected RoleAssigned change"),
        }
    }

    #[test]
    fn given_duplicate_role_then_assign_does_not_record_event() {
        // arrange
        let clock = MockClock::new(Utc::now());
        let role_id = ProjectRoleId::new();
        let mut pm = ProjectMember::new(MemberId::new(), ProjectId::new(), vec![role_id.clone()], &clock);
        let _ = pm.pull_changes();

        // act
        pm.assign_role(role_id);

        // assert
        assert!(pm.pull_changes().is_empty());
    }

    #[test]
    fn given_existing_role_then_remove_records_role_removed_event() {
        // arrange
        let clock = MockClock::new(Utc::now());
        let role_id = ProjectRoleId::new();
        let mut pm = ProjectMember::new(MemberId::new(), ProjectId::new(), vec![role_id.clone()], &clock);
        let _ = pm.pull_changes();

        // act
        pm.remove_role(&role_id);

        // assert
        assert!(pm.role_ids().is_empty());

        let changes = pm.pull_changes();
        assert_eq!(changes.len(), 1);
        match &changes[0] {
            ProjectMemberChange::RoleRemoved { id, role_id: rid } => {
                assert_eq!(id, pm.id());
                assert_eq!(rid, &role_id);
            },
            _ => panic!("Expected RoleRemoved change"),
        }
    }

    #[test]
    fn given_nonexistent_role_then_remove_does_not_record_event() {
        // arrange
        let clock = MockClock::new(Utc::now());
        let mut pm = ProjectMember::new(MemberId::new(), ProjectId::new(), vec![ProjectRoleId::new()], &clock);
        let _ = pm.pull_changes();

        // act
        pm.remove_role(&ProjectRoleId::new());

        // assert
        assert!(pm.pull_changes().is_empty());
    }
}
