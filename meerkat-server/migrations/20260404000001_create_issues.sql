CREATE TABLE issues (
    id                UUID PRIMARY KEY,
    project_id        UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    title             TEXT NOT NULL,
    fingerprint_hash  TEXT NOT NULL,
    status            TEXT NOT NULL DEFAULT 'unresolved',
    level             TEXT NOT NULL DEFAULT 'error',
    event_count       BIGINT NOT NULL DEFAULT 1,
    first_seen        TIMESTAMPTZ NOT NULL,
    last_seen         TIMESTAMPTZ NOT NULL,
    version           BIGINT NOT NULL DEFAULT 1,
    created_at        TIMESTAMPTZ NOT NULL,
    updated_at        TIMESTAMPTZ NOT NULL
);

CREATE UNIQUE INDEX idx_issues_project_fingerprint ON issues (project_id, fingerprint_hash);
CREATE INDEX idx_issues_project_id ON issues (project_id);
