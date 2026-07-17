CREATE TABLE IF NOT EXISTS actors (
    id              UUID        PRIMARY KEY,
    tenant_id       UUID        NOT NULL,
    external_id     TEXT        NOT NULL,
    role            TEXT        NOT NULL, -- user, agent
    name            TEXT        NOT NULL, -- user_name
    display_name    TEXT        NOT NULL, -- User Name
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    UNIQUE (tenant_id, external_id)
);
