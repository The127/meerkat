ALTER TABLE issues ADD COLUMN issue_number BIGINT NOT NULL DEFAULT 0;
CREATE UNIQUE INDEX idx_issues_project_issue_number ON issues(project_id, issue_number);
