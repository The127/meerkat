ALTER TABLE oidc_configs
    ADD COLUMN sub_claim     TEXT NOT NULL DEFAULT 'sub',
    ADD COLUMN name_claim    TEXT NOT NULL DEFAULT 'preferred_username',
    ADD COLUMN role_claim    TEXT NOT NULL DEFAULT 'roles',
    ADD COLUMN owner_values  TEXT[] NOT NULL DEFAULT '{}',
    ADD COLUMN admin_values  TEXT[] NOT NULL DEFAULT '{}',
    ADD COLUMN member_values TEXT[] NOT NULL DEFAULT '{}';

ALTER TABLE oidc_configs
    ALTER COLUMN sub_claim DROP DEFAULT,
    ALTER COLUMN name_claim DROP DEFAULT,
    ALTER COLUMN role_claim DROP DEFAULT,
    ALTER COLUMN owner_values DROP DEFAULT,
    ALTER COLUMN admin_values DROP DEFAULT,
    ALTER COLUMN member_values DROP DEFAULT;
