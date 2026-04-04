CREATE TABLE events (
    id                UUID PRIMARY KEY,
    project_id        UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    issue_id          UUID NOT NULL REFERENCES issues(id) ON DELETE CASCADE,
    fingerprint_hash  TEXT NOT NULL,
    message           TEXT NOT NULL,
    level             TEXT NOT NULL DEFAULT 'error',
    platform          TEXT NOT NULL,
    timestamp         TIMESTAMPTZ NOT NULL,
    server_name       TEXT,
    environment       TEXT,
    release           TEXT,
    exception_type    TEXT,
    exception_value   TEXT,
    tags              JSONB NOT NULL DEFAULT '[]',
    extra             JSONB NOT NULL DEFAULT '{}',
    created_at        TIMESTAMPTZ NOT NULL,
    updated_at        TIMESTAMPTZ NOT NULL
);

CREATE INDEX idx_events_project_id ON events (project_id);
CREATE INDEX idx_events_issue_id ON events (issue_id);
CREATE INDEX idx_events_timestamp ON events (timestamp);
