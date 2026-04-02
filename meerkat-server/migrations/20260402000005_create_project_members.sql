CREATE TABLE project_members (
    id         UUID PRIMARY KEY,
    member_id  UUID NOT NULL REFERENCES members(id) ON DELETE CASCADE,
    project_id UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL,
    UNIQUE (member_id, project_id)
);

CREATE TABLE project_member_roles (
    project_member_id UUID NOT NULL REFERENCES project_members(id) ON DELETE CASCADE,
    role_id           UUID NOT NULL REFERENCES project_roles(id) ON DELETE CASCADE,
    PRIMARY KEY (project_member_id, role_id)
);
