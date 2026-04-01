use chrono::{DateTime, Utc};
use meerkat_macros::{uuid_id, slug_id, Reconstitute};
use crate::shared::version::Version;
use crate::shared::change_tracker::ChangeTracker;
use crate::ports::clock::Clock;
use crate::models::oidc_config::{OidcConfig, OidcConfigId, OidcConfigStatus};

uuid_id!(OrganizationId);
slug_id!(OrganizationSlug);

#[derive(Debug, Clone, Reconstitute)]
pub struct Organization {
    id: OrganizationId,
    name: String,
    slug: OrganizationSlug,
    oidc_configs: Vec<OidcConfig>,
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
        initial_oidc_config: OidcConfig,
    },
    NameUpdated {
        id: OrganizationId,
        old_name: String,
        new_name: String,
    },
    OidcConfigAdded {
        org_id: OrganizationId,
        config: OidcConfig,
    },
    ActiveOidcConfigSwitched {
        org_id: OrganizationId,
        old_config_id: OidcConfigId,
        new_config_id: OidcConfigId,
    },
    OidcConfigDeleted {
        org_id: OrganizationId,
        config_id: OidcConfigId,
    },
    Deleted {
        id: OrganizationId,
    },
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
        let name_str = name.to_string();

        let change = OrganizationChange::Created {
            id: id.clone(),
            name: name_str.clone(),
            slug: slug.clone(),
            initial_oidc_config: oidc_config.clone(),
        };

        let now = clock.now();
        let mut changes = ChangeTracker::new();
        changes.record(change);

        Ok(Organization {
            id,
            name: name_str,
            slug,
            oidc_configs: vec![oidc_config],
            created_at: now,
            updated_at: now,
            version: Version::initial(),
            changes,
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

        let old_name = self.name.clone();
        let new_name_str = new_name.to_string();

        self.changes.record(OrganizationChange::NameUpdated {
            id: self.id.clone(),
            old_name,
            new_name: new_name_str.clone(),
        });

        self.name = new_name_str;

        Ok(())
    }

    pub fn add_draft_oidc_config(
        &mut self,
        config: OidcConfig,
    ) -> Result<(), OrganizationError> {
        if config.status() != &OidcConfigStatus::Draft {
            return Err(OrganizationError::OidcConfigMustBeDraft);
        }

        self.changes.record(OrganizationChange::OidcConfigAdded {
            org_id: self.id.clone(),
            config: config.clone(),
        });

        self.oidc_configs.push(config);

        Ok(())
    }

    pub fn switch_active_oidc_config(
        &mut self,
        config_id: &OidcConfigId,
    ) -> Result<(), OrganizationError> {
        let current_config = self.oidc_configs.iter_mut().find(|c| c.is_active())
            .expect("organization invariant violated: no active OIDC config");
        let old_config_id = current_config.id().clone();

        if current_config.id() == config_id {
            return Ok(())
        }

        current_config.deactivate()?;

        let target_config = self.oidc_configs.iter_mut().find(|c| c.id() == config_id)
            .ok_or(OrganizationError::OidcConfigNotFound)?;

        target_config.activate()?;

        self.changes.record(OrganizationChange::ActiveOidcConfigSwitched {
            org_id: self.id.clone(),
            old_config_id,
            new_config_id: config_id.clone(),
        });

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

        let before = self.oidc_configs.len();
        self.oidc_configs.retain(|c| c.id() != config_id);

        if self.oidc_configs.len() < before {
            self.changes.record(OrganizationChange::OidcConfigDeleted {
                org_id: self.id.clone(),
                config_id: config_id.clone(),
            });
        }

        Ok(())
    }

    pub fn pull_changes(&mut self) -> Vec<OrganizationChange> {
        self.changes.pull_changes()
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
    fn given_valid_name_and_slug_organization_creation_should_succeed_and_record_creation_event() {
        // arrange
        let slug = OrganizationSlug::new("meerkat-inc").unwrap();
        let expected_now = Utc::now();
        let clock = MockClock::new(expected_now);
        let oidc_config = draft_config("Default SSO", &clock);

        // act
        let mut org = Organization::new("Meerkat Inc.".into(), slug.clone(), oidc_config, &clock)
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

        let changes = org.pull_changes();
        assert_eq!(changes.len(), 1);
        match &changes[0] {
            OrganizationChange::Created { id, name: event_name, slug: event_slug, initial_oidc_config } => {
                assert_eq!(id, org.id());
                assert_eq!(event_name, "Meerkat Inc.");
                assert_eq!(event_slug, &slug);
                assert!(initial_oidc_config.is_active());
            },
            _ => panic!("Expected Created change"),
        }
    }

    #[test]
    fn given_an_empty_name_organization_creation_should_fail_with_an_empty_name_error() {
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
    fn given_a_name_with_extra_spaces_organization_creation_should_trim_the_name() {
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
    fn given_an_existing_organization_updating_its_name_should_succeed_and_record_change_event() {
        // arrange
        let (mut org, _) = test_org();
        let _ = org.pull_changes();

        // act
        org.update_name("New Name".into()).expect("Failed to update organization name");

        // assert
        assert_eq!(org.name(), "New Name");
        assert_eq!(org.version(), &Version::initial());

        let changes = org.pull_changes();
        assert_eq!(changes.len(), 1);
        match &changes[0] {
            OrganizationChange::NameUpdated { id, old_name, new_name } => {
                assert_eq!(id, org.id());
                assert_eq!(old_name, "Test Org");
                assert_eq!(new_name, "New Name");
            },
            _ => panic!("Expected NameUpdated change"),
        }
    }

    #[test]
    fn given_an_existing_organization_updating_its_name_to_empty_should_fail() {
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
    fn given_the_same_name_updating_organization_name_should_do_nothing() {
        // arrange
        let (mut org, _) = test_org();
        let _ = org.pull_changes();

        // act
        org.update_name("Test Org".into()).expect("Update should succeed");

        // assert
        assert_eq!(org.name(), "Test Org");
        assert_eq!(org.version(), &Version::initial());
        assert!(org.pull_changes().is_empty());
    }

    // --- add_draft_oidc_config ---

    #[test]
    fn given_a_draft_config_adding_it_should_succeed_and_record_oidc_config_added_event() {
        // arrange
        let (mut org, clock) = test_org();
        let _ = org.pull_changes();
        let new_config = draft_config("Secondary SSO", &clock);
        let expected_id = new_config.id().clone();

        // act
        org.add_draft_oidc_config(new_config)
            .expect("Failed to add draft OIDC config");

        // assert
        assert_eq!(org.oidc_configs().len(), 2);

        let added = org.oidc_configs().iter().find(|c| c.id() == &expected_id).unwrap();
        assert_eq!(added.status(), &OidcConfigStatus::Draft);

        let changes = org.pull_changes();
        assert_eq!(changes.len(), 1);
        match &changes[0] {
            OrganizationChange::OidcConfigAdded { org_id, config } => {
                assert_eq!(org_id, org.id());
                assert_eq!(config.id(), &expected_id);
            },
            _ => panic!("Expected OidcConfigAdded change"),
        }
    }

    #[test]
    fn given_a_non_draft_config_adding_it_should_fail_with_oidc_config_must_be_draft() {
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
    fn given_a_draft_config_switching_to_it_should_activate_it_and_deactivate_the_old_one() {
        // arrange
        let (mut org, clock) = test_org();
        let old_active_id = org.oidc_configs()[0].id().clone();
        let new_config = draft_config("New SSO", &clock);
        let new_config_id = new_config.id().clone();
        org.add_draft_oidc_config(new_config).unwrap();
        let _ = org.pull_changes();

        // act
        org.switch_active_oidc_config(&new_config_id)
            .expect("Failed to switch active OIDC config");

        // assert
        let old = org.oidc_configs().iter().find(|c| c.id() == &old_active_id).unwrap();
        assert_eq!(old.status(), &OidcConfigStatus::Inactive);

        let new = org.oidc_configs().iter().find(|c| c.id() == &new_config_id).unwrap();
        assert_eq!(new.status(), &OidcConfigStatus::Active);

        let changes = org.pull_changes();
        assert_eq!(changes.len(), 1);
        match &changes[0] {
            OrganizationChange::ActiveOidcConfigSwitched { org_id, old_config_id, new_config_id: event_new_id } => {
                assert_eq!(org_id, org.id());
                assert_eq!(old_config_id, &old_active_id);
                assert_eq!(event_new_id, &new_config_id);
            },
            _ => panic!("Expected ActiveOidcConfigSwitched change"),
        }
    }

    #[test]
    fn given_a_nonexistent_config_id_switching_should_fail_with_oidc_config_not_found() {
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
    fn given_the_already_active_config_switching_should_do_nothing() {
        // arrange
        let (mut org, _) = test_org();
        let active_id = org.oidc_configs()[0].id().clone();
        let _ = org.pull_changes();

        // act
        org.switch_active_oidc_config(&active_id)
            .expect("Switch should succeed");

        // assert
        assert!(org.oidc_configs()[0].is_active());
        assert!(org.pull_changes().is_empty());
    }

    // --- delete_oidc_config ---

    #[test]
    fn given_an_inactive_config_deleting_it_should_remove_it_and_record_oidc_config_deleted_event() {
        // arrange
        let (mut org, clock) = test_org();
        let draft = draft_config("To Delete", &clock);
        let draft_id = draft.id().clone();
        org.add_draft_oidc_config(draft).unwrap();
        let _ = org.pull_changes();

        // act
        org.delete_oidc_config(&draft_id)
            .expect("Failed to delete OIDC config");

        // assert
        assert_eq!(org.oidc_configs().len(), 1);
        assert!(org.oidc_configs().iter().all(|c| c.id() != &draft_id));

        let changes = org.pull_changes();
        assert_eq!(changes.len(), 1);
        match &changes[0] {
            OrganizationChange::OidcConfigDeleted { org_id, config_id } => {
                assert_eq!(org_id, org.id());
                assert_eq!(config_id, &draft_id);
            },
            _ => panic!("Expected OidcConfigDeleted change"),
        }
    }

    #[test]
    fn given_the_active_config_deleting_it_should_fail_with_cannot_delete_active_config() {
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
