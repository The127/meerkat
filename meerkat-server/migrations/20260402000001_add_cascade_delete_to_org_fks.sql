ALTER TABLE projects
    DROP CONSTRAINT projects_organization_id_fkey,
    ADD CONSTRAINT projects_organization_id_fkey
        FOREIGN KEY (organization_id) REFERENCES organizations(id) ON DELETE CASCADE;

ALTER TABLE oidc_configs
    DROP CONSTRAINT oidc_configs_organization_id_fkey,
    ADD CONSTRAINT oidc_configs_organization_id_fkey
        FOREIGN KEY (organization_id) REFERENCES organizations(id) ON DELETE CASCADE;
