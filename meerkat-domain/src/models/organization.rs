use chrono::{DateTime, Utc};
use meerkat_macros::{uuid_id, slug_id, Reconstitute};
use crate::shared::version::Version;
use crate::ports::clock::Clock;
use crate::models::oidc_config::{ClaimMapping, OidcConfig, OidcConfigId, OidcConfigStatus};

uuid_id!(OrganizationId);
slug_id!(OrganizationSlug);

#[derive(Debug, Clone)]
pub enum OrganizationIdentifier {
    Id(OrganizationId),
    Slug(OrganizationSlug),
}

impl From<OrganizationId> for OrganizationIdentifier {
    fn from(id: OrganizationId) -> Self { Self::Id(id) }
}

impl From<OrganizationSlug> for OrganizationIdentifier {
    fn from(slug: OrganizationSlug) -> Self { Self::Slug(slug) }
}

#[derive(Debug, Clone, Reconstitute)]
pub struct Organization {
    id: OrganizationId,
    name: String,
    slug: OrganizationSlug,
    oidc_configs: Vec<OidcConfig>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    version: Version,
}

#[derive(Debug, thiserror::Error)]
pub enum OrganizationError {
    #[error("organization name must not be empty")]
    EmptyName,
    #[error("OIDC config error: {0}")]
    OidcConfig(#[from] crate::models::oidc_config::OidcConfigError),
    #[error("OIDC config not found")]
    OidcConfigNotFound,
    #[error("cannot delete an active OIDC config")]
    CannotDeleteActiveConfig,
    #[error("OIDC config must be in draft status to be added")]
    OidcConfigMustBeDraft,
}

impl Organization {
    pub fn new(
        name: String,
        slug: OrganizationSlug,
        mut oidc_config: OidcConfig,
        clock: &dyn Clock,
    ) -> Result<Self, OrganizationError> {
        let name = name.trim();
        if name.is_empty() {
            return Err(OrganizationError::EmptyName);
        }

        oidc_config.activate()?;

        let id = OrganizationId::new();
        let now = clock.now();

        Ok(Organization {
            id,
            name: name.to_string(),
            slug,
            oidc_configs: vec![oidc_config],
            created_at: now,
            updated_at: now,
            version: Version::initial(),
        })
    }

    pub fn update_name(&mut self, new_name: String) -> Result<(), OrganizationError> {
        let new_name = new_name.trim();
        if new_name.is_empty() {
            return Err(OrganizationError::EmptyName);
        }

        if new_name == self.name {
            return Ok(());
        }

        self.name = new_name.to_string();
        Ok(())
    }

    pub fn add_draft_oidc_config(
        &mut self,
        config: OidcConfig,
    ) -> Result<(), OrganizationError> {
        if config.status() != &OidcConfigStatus::Draft {
            return Err(OrganizationError::OidcConfigMustBeDraft);
        }

        self.oidc_configs.push(config);
        Ok(())
    }

    pub fn switch_active_oidc_config(
        &mut self,
        config_id: &OidcConfigId,
    ) -> Result<(), OrganizationError> {
        let current_config = self.oidc_configs.iter_mut().find(|c| c.is_active())
            .expect("organization invariant violated: no active OIDC config");

        if current_config.id() == config_id {
            return Ok(())
        }

        current_config.deactivate()?;

        let target_config = self.oidc_configs.iter_mut().find(|c| c.id() == config_id)
            .ok_or(OrganizationError::OidcConfigNotFound)?;

        target_config.activate()?;
        Ok(())
    }

    pub fn update_oidc_config_claim_mapping(
        &mut self,
        config_id: &OidcConfigId,
        claim_mapping: ClaimMapping,
    ) -> Result<(), OrganizationError> {
        let config = self.oidc_configs.iter_mut()
            .find(|c| c.id() == config_id)
            .ok_or(OrganizationError::OidcConfigNotFound)?;

        config.update_claim_mapping(claim_mapping);
        Ok(())
    }

    pub fn delete_oidc_config(
        &mut self,
        config_id: &OidcConfigId,
    ) -> Result<(), OrganizationError> {
        let active_id = self.oidc_configs.iter().find(|c| c.is_active())
            .expect("organization invariant violated: no active OIDC config")
            .id();

        if active_id == config_id {
            return Err(OrganizationError::CannotDeleteActiveConfig);
        }

        self.oidc_configs.retain(|c| c.id() != config_id);
        Ok(())
    }

    pub fn id(&self) -> &OrganizationId { &self.id }
    pub fn name(&self) -> &str { &self.name }
    pub fn slug(&self) -> &OrganizationSlug { &self.slug }
    pub fn oidc_configs(&self) -> &[OidcConfig] { &self.oidc_configs }
    pub fn created_at(&self) -> &DateTime<Utc> { &self.created_at }
    pub fn updated_at(&self) -> &DateTime<Utc> { &self.updated_at }
    pub fn version(&self) -> &Version { &self.version }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ports::clock::MockClock;
    use crate::testing::{draft_config, test_org};

    #[test]
    fn given_valid_name_and_slug_then_organization_creation_succeeds() {
        // arrange
        let slug = OrganizationSlug::new("meerkat-inc").unwrap();
        let expected_now = Utc::now();
        let clock = MockClock::new(expected_now);
        let oidc_config = draft_config("Default SSO", &clock);

        // act
        let org = Organization::new("Meerkat Inc.".into(), slug.clone(), oidc_config, &clock)
            .expect("Failed to create organization");

        // assert
        assert_eq!(org.name(), "Meerkat Inc.");
        assert_eq!(org.slug(), &slug);
        assert_eq!(org.version(), &Version::initial());
        assert!(!org.id().as_uuid().is_nil());
        assert_eq!(org.created_at(), &expected_now);
        assert_eq!(org.updated_at(), &expected_now);
        assert_eq!(org.oidc_configs().len(), 1);
        assert!(org.oidc_configs()[0].is_active());
    }

    #[test]
    fn given_an_empty_name_then_organization_creation_fails() {
        // arrange
        let clock = MockClock::new(Utc::now());
        let slug = OrganizationSlug::new("empty-name").unwrap();

        // act
        let result = Organization::new("  ".into(), slug, draft_config("Default SSO", &clock), &clock);

        // assert
        match result {
            Err(OrganizationError::EmptyName) => (),
            _ => panic!("Expected EmptyName error, got {:?}", result),
        }
    }

    #[test]
    fn given_a_name_with_extra_spaces_then_organization_creation_trims() {
        // arrange
        let clock = MockClock::new(Utc::now());
        let slug = OrganizationSlug::new("meerkat-inc").unwrap();

        // act
        let org = Organization::new("  Meerkat Inc.  ".into(), slug, draft_config("Default SSO", &clock), &clock)
            .expect("Failed to create organization");

        // assert
        assert_eq!(org.name(), "Meerkat Inc.");
    }

    #[test]
    fn given_existing_org_then_updating_name_succeeds() {
        // arrange
        let (mut org, _) = test_org();

        // act
        org.update_name("New Name".into()).expect("Failed to update organization name");

        // assert
        assert_eq!(org.name(), "New Name");
    }

    #[test]
    fn given_empty_name_then_updating_name_fails() {
        // arrange
        let (mut org, _) = test_org();

        // act
        let result = org.update_name("  ".into());

        // assert
        match result {
            Err(OrganizationError::EmptyName) => (),
            _ => panic!("Expected EmptyName error, got {:?}", result),
        }
    }

    #[test]
    fn given_same_name_then_updating_name_is_idempotent() {
        // arrange
        let (mut org, _) = test_org();

        // act
        org.update_name("Test Org".into()).expect("Update should succeed");

        // assert
        assert_eq!(org.name(), "Test Org");
    }

    // --- add_draft_oidc_config ---

    #[test]
    fn given_a_draft_config_then_adding_it_succeeds() {
        // arrange
        let (mut org, clock) = test_org();
        let new_config = draft_config("Secondary SSO", &clock);

        // act
        org.add_draft_oidc_config(new_config)
            .expect("Failed to add draft OIDC config");

        // assert
        assert_eq!(org.oidc_configs().len(), 2);
    }

    #[test]
    fn given_a_non_draft_config_then_adding_it_fails() {
        // arrange
        let (mut org, clock) = test_org();
        let mut active_config = draft_config("Already Active", &clock);
        active_config.activate().unwrap();

        // act
        let result = org.add_draft_oidc_config(active_config);

        // assert
        match result {
            Err(OrganizationError::OidcConfigMustBeDraft) => (),
            _ => panic!("Expected OidcConfigMustBeDraft error, got {:?}", result),
        }
    }

    // --- switch_active_oidc_config ---

    #[test]
    fn given_a_draft_config_then_switching_to_it_activates_it_and_deactivates_old() {
        // arrange
        let (mut org, clock) = test_org();
        let old_active_id = org.oidc_configs()[0].id().clone();
        let new_config = draft_config("New SSO", &clock);
        let new_config_id = new_config.id().clone();
        org.add_draft_oidc_config(new_config).unwrap();

        // act
        org.switch_active_oidc_config(&new_config_id)
            .expect("Failed to switch active OIDC config");

        // assert
        let old = org.oidc_configs().iter().find(|c| c.id() == &old_active_id).unwrap();
        assert_eq!(old.status(), &OidcConfigStatus::Inactive);

        let new = org.oidc_configs().iter().find(|c| c.id() == &new_config_id).unwrap();
        assert_eq!(new.status(), &OidcConfigStatus::Active);
    }

    #[test]
    fn given_nonexistent_config_id_then_switching_fails() {
        // arrange
        let (mut org, _) = test_org();
        let nonexistent_id = OidcConfigId::new();

        // act
        let result = org.switch_active_oidc_config(&nonexistent_id);

        // assert
        match result {
            Err(OrganizationError::OidcConfigNotFound) => (),
            _ => panic!("Expected OidcConfigNotFound error, got {:?}", result),
        }
    }

    #[test]
    fn given_already_active_config_then_switching_is_idempotent() {
        // arrange
        let (mut org, _) = test_org();
        let active_id = org.oidc_configs()[0].id().clone();

        // act
        org.switch_active_oidc_config(&active_id)
            .expect("Switch should succeed");

        // assert
        assert!(org.oidc_configs()[0].is_active());
    }

    // --- delete_oidc_config ---

    #[test]
    fn given_inactive_config_then_deleting_it_removes_it() {
        // arrange
        let (mut org, clock) = test_org();
        let draft = draft_config("To Delete", &clock);
        let draft_id = draft.id().clone();
        org.add_draft_oidc_config(draft).unwrap();

        // act
        org.delete_oidc_config(&draft_id)
            .expect("Failed to delete OIDC config");

        // assert
        assert_eq!(org.oidc_configs().len(), 1);
        assert!(org.oidc_configs().iter().all(|c| c.id() != &draft_id));
    }

    #[test]
    fn given_active_config_then_deleting_it_fails() {
        // arrange
        let (mut org, _) = test_org();
        let active_id = org.oidc_configs()[0].id().clone();

        // act
        let result = org.delete_oidc_config(&active_id);

        // assert
        match result {
            Err(OrganizationError::CannotDeleteActiveConfig) => (),
            _ => panic!("Expected CannotDeleteActiveConfig error, got {:?}", result),
        }
    }
}
