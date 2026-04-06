use meerkat_macros::{uuid_id, slug_id, Reconstitute};
use vec1::Vec1;
use crate::models::permission::ProjectPermission;
use crate::models::project::ProjectId;

uuid_id!(ProjectRoleId);
slug_id!(ProjectRoleSlug);

#[derive(Debug, Clone)]
pub enum ProjectRoleIdentifier {
    Id(ProjectRoleId),
    Slug(ProjectId, ProjectRoleSlug),
}

#[derive(Debug, Clone, Reconstitute)]
pub struct ProjectRole {
    id: ProjectRoleId,
    project_id: ProjectId,
    name: String,
    slug: ProjectRoleSlug,
    permissions: Vec1<ProjectPermission>,
    is_default: bool,
}

#[derive(Debug, thiserror::Error)]
pub enum ProjectRoleError {
    #[error("project role name must not be empty")]
    EmptyName,
}

impl ProjectRole {
    pub fn new(
        project_id: ProjectId,
        name: String,
        slug: ProjectRoleSlug,
        permissions: Vec1<ProjectPermission>,
        is_default: bool,
    ) -> Result<Self, ProjectRoleError> {
        let name = name.trim().to_string();
        if name.is_empty() {
            return Err(ProjectRoleError::EmptyName);
        }

        Ok(Self {
            id: ProjectRoleId::new(),
            project_id,
            name,
            slug,
            permissions,
            is_default,
        })
    }

    pub fn default_roles(project_id: ProjectId) -> (Vec<ProjectRole>, ProjectRoleId) {
        let admin_id = ProjectRoleId::new();

        let roles = vec![
            ProjectRole {
                id: ProjectRoleId::new(),
                project_id: project_id.clone(),
                name: "Viewer".to_string(),
                slug: ProjectRoleSlug::new("viewer").unwrap(),
                permissions: Vec1::new(ProjectPermission::ProjectRead),
                is_default: true,
            },
            ProjectRole {
                id: ProjectRoleId::new(),
                project_id: project_id.clone(),
                name: "Editor".to_string(),
                slug: ProjectRoleSlug::new("editor").unwrap(),
                permissions: Vec1::try_from_vec(vec![
                    ProjectPermission::ProjectRead,
                    ProjectPermission::ProjectWrite,
                ]).unwrap(),
                is_default: true,
            },
            ProjectRole {
                id: admin_id.clone(),
                project_id,
                name: "Admin".to_string(),
                slug: ProjectRoleSlug::new("admin").unwrap(),
                permissions: Vec1::try_from_vec(vec![
                    ProjectPermission::ProjectRead,
                    ProjectPermission::ProjectWrite,
                    ProjectPermission::ProjectDelete,
                    ProjectPermission::ProjectManageMembers,
                    ProjectPermission::ProjectManageKeys,
                ]).unwrap(),
                is_default: true,
            },
        ];

        (roles, admin_id)
    }

    pub fn update(&mut self, name: String, permissions: Vec1<ProjectPermission>) -> Result<(), ProjectRoleError> {
        let name = name.trim().to_string();
        if name.is_empty() {
            return Err(ProjectRoleError::EmptyName);
        }
        self.name = name;
        self.permissions = permissions;
        Ok(())
    }

    pub fn id(&self) -> &ProjectRoleId { &self.id }
    pub fn project_id(&self) -> &ProjectId { &self.project_id }
    pub fn name(&self) -> &str { &self.name }
    pub fn slug(&self) -> &ProjectRoleSlug { &self.slug }
    pub fn permissions(&self) -> &Vec1<ProjectPermission> { &self.permissions }
    pub fn is_default(&self) -> bool { self.is_default }
}

#[cfg(test)]
mod tests {
    use super::*;
    use vec1::vec1;

    #[test]
    fn given_valid_input_then_creation_succeeds() {
        // arrange
        let project_id = ProjectId::new();

        // act
        let role = ProjectRole::new(
            project_id.clone(),
            "Custom Role".into(),
            ProjectRoleSlug::new("custom-role").unwrap(),
            vec1![ProjectPermission::ProjectRead],
            false,
        ).unwrap();

        // assert
        assert_eq!(role.name(), "Custom Role");
        assert_eq!(role.slug().as_str(), "custom-role");
        assert_eq!(role.project_id(), &project_id);
        assert_eq!(role.permissions(), &vec1![ProjectPermission::ProjectRead]);
        assert!(!role.is_default());
    }

    #[test]
    fn given_empty_name_then_creation_fails() {
        // act
        let result = ProjectRole::new(
            ProjectId::new(),
            "  ".into(),
            ProjectRoleSlug::new("empty").unwrap(),
            vec1![ProjectPermission::ProjectRead],
            false,
        );

        // assert
        match result {
            Err(ProjectRoleError::EmptyName) => (),
            _ => panic!("Expected EmptyName error, got {:?}", result),
        }
    }

    #[test]
    fn given_valid_update_then_name_and_permissions_change() {
        // arrange
        let mut role = ProjectRole::new(
            ProjectId::new(),
            "Original".into(),
            ProjectRoleSlug::new("original").unwrap(),
            vec1![ProjectPermission::ProjectRead],
            false,
        ).unwrap();

        // act
        role.update("Updated".into(), vec1![ProjectPermission::ProjectRead, ProjectPermission::ProjectWrite]).unwrap();

        // assert
        assert_eq!(role.name(), "Updated");
        assert_eq!(role.permissions().len(), 2);
    }

    #[test]
    fn given_empty_name_on_update_then_fails() {
        // arrange
        let mut role = ProjectRole::new(
            ProjectId::new(),
            "Original".into(),
            ProjectRoleSlug::new("original").unwrap(),
            vec1![ProjectPermission::ProjectRead],
            false,
        ).unwrap();

        // act
        let result = role.update("  ".into(), vec1![ProjectPermission::ProjectRead]);

        // assert
        assert!(matches!(result, Err(ProjectRoleError::EmptyName)));
    }

    #[test]
    fn given_project_id_then_default_roles_creates_three_roles() {
        // arrange
        let project_id = ProjectId::new();

        // act
        let (roles, admin_role_id) = ProjectRole::default_roles(project_id.clone());

        // assert
        assert_eq!(roles.len(), 3);

        let viewer = &roles[0];
        assert_eq!(viewer.name(), "Viewer");
        assert_eq!(viewer.slug().as_str(), "viewer");
        assert!(viewer.is_default());
        assert_eq!(viewer.permissions(), &vec1![ProjectPermission::ProjectRead]);
        assert_eq!(viewer.project_id(), &project_id);

        let editor = &roles[1];
        assert_eq!(editor.name(), "Editor");
        assert_eq!(editor.slug().as_str(), "editor");
        assert!(editor.is_default());
        assert_eq!(editor.permissions().len(), 2);

        let admin = &roles[2];
        assert_eq!(admin.name(), "Admin");
        assert_eq!(admin.slug().as_str(), "admin");
        assert!(admin.is_default());
        assert_eq!(admin.permissions().len(), 5);
        assert!(admin.permissions().contains(&ProjectPermission::ProjectManageMembers));
        assert!(admin.permissions().contains(&ProjectPermission::ProjectManageKeys));
        assert_eq!(admin.id(), &admin_role_id);
    }
}
