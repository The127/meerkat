CREATE TABLE project_keys (
    id          UUID PRIMARY KEY,
    project_id  UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    key_token   TEXT NOT NULL,
    label       TEXT NOT NULL,
    status      TEXT NOT NULL DEFAULT 'active',
    created_at  TIMESTAMPTZ NOT NULL,
    updated_at  TIMESTAMPTZ NOT NULL,
    version     BIGINT NOT NULL DEFAULT 0
);

CREATE UNIQUE INDEX idx_project_keys_key_token ON project_keys (key_token);
CREATE INDEX idx_project_keys_project_id ON project_keys (project_id);
