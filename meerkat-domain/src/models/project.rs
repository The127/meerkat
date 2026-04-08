use raccoon_typed_id::{uuid_id, slug_id};
use reconstitute::Reconstitute;
use crate::shared::version::Version;
use crate::models::organization::OrganizationId;

uuid_id!(ProjectId);
slug_id!(ProjectSlug);

#[derive(Debug, Clone)]
pub enum ProjectIdentifier {
    Id(ProjectId),
    Slug(OrganizationId, ProjectSlug),
}

#[derive(Debug, Clone, Reconstitute)]
pub struct Project {
    id: ProjectId,
    organization_id: OrganizationId,
    name: String,
    slug: ProjectSlug,
    version: Version,
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
    ) -> Result<Self, ProjectError> {
        let name = name.trim();
        if name.is_empty() {
            return Err(ProjectError::EmptyName);
        }

        let id = ProjectId::new();

        Ok(Project {
            id,
            organization_id,
            name: name.to_string(),
            slug,
            version: Version::initial(),
        })
    }

    pub fn update_name(&mut self, new_name: String) -> Result<(), ProjectError> {
        let new_name = new_name.trim();
        if new_name.is_empty() {
            return Err(ProjectError::EmptyName);
        }

        if new_name == self.name {
            return Ok(());
        }

        self.name = new_name.to_string();

        Ok(())
    }

    pub fn id(&self) -> &ProjectId { &self.id }
    pub fn organization_id(&self) -> &OrganizationId { &self.organization_id }
    pub fn name(&self) -> &str { &self.name }
    pub fn slug(&self) -> &ProjectSlug { &self.slug }
    pub fn version(&self) -> &Version { &self.version }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::test_project;

    #[test]
    fn given_valid_input_then_project_creation_succeeds() {
        // arrange
        let org_id = OrganizationId::new();
        let slug = ProjectSlug::new("my-project").unwrap();

        // act
        let project = Project::new(org_id.clone(), "My Project".into(), slug.clone())
            .expect("Failed to create project");

        // assert
        assert_eq!(project.name(), "My Project");
        assert_eq!(project.slug(), &slug);
        assert_eq!(project.organization_id(), &org_id);
        assert_eq!(project.version(), &Version::initial());
    }

    #[test]
    fn given_empty_name_then_project_creation_fails() {
        // arrange
        let slug = ProjectSlug::new("some-slug").unwrap();

        // act
        let result = Project::new(OrganizationId::new(), "  ".into(), slug);

        // assert
        match result {
            Err(ProjectError::EmptyName) => (),
            other => panic!("Expected EmptyName error, got {:?}", other),
        }
    }

    #[test]
    fn given_existing_project_then_updating_name_succeeds() {
        // arrange
        let mut project = test_project();

        // act
        project.update_name("New Name".into()).unwrap();

        // assert
        assert_eq!(project.name(), "New Name");
    }

    #[test]
    fn given_same_name_then_updating_does_nothing() {
        // arrange
        let mut project = test_project();

        // act
        project.update_name("Test Project".into()).unwrap();

        // assert
        assert_eq!(project.name(), "Test Project");
    }
}
