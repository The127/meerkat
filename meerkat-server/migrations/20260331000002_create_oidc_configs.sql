CREATE TABLE oidc_configs (
    id              UUID PRIMARY KEY,
    organization_id UUID NOT NULL REFERENCES organizations(id),
    name            TEXT NOT NULL,
    client_id       TEXT NOT NULL,
    issuer_url      TEXT NOT NULL,
    audience        TEXT NOT NULL,
    jwks_url        TEXT NULL,
    status          TEXT NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL,
    updated_at      TIMESTAMPTZ NOT NULL
);
