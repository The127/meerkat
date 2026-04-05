CREATE TABLE oidc_config_warnings (
    oidc_config_id   UUID NOT NULL REFERENCES oidc_configs(id) ON DELETE CASCADE,
    warning_key      TEXT NOT NULL,
    message          TEXT NOT NULL,
    context          JSONB,
    first_seen       TIMESTAMPTZ NOT NULL,
    last_seen        TIMESTAMPTZ NOT NULL,
    occurrence_count BIGINT NOT NULL DEFAULT 1,
    PRIMARY KEY (oidc_config_id, warning_key)
);
