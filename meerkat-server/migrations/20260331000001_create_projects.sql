CREATE TABLE projects (
    id              UUID        PRIMARY KEY,
    organization_id UUID        NOT NULL REFERENCES organizations(id),
    name            TEXT        NOT NULL,
    slug            TEXT        NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL,
    updated_at      TIMESTAMPTZ NOT NULL,
    version         BIGINT      NOT NULL DEFAULT 1,
    UNIQUE (organization_id, slug)
);
